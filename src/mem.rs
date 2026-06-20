use crossbeam::channel;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{
        Color,
        Style,
    },
    widgets::Widget,
};

use crate::{
    block::Block,
    cache_aligned,
    cfg::{
        CONFIG,
        CacheLine,
        Word,
    },
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(dram);

pub enum MemoryOps {
    Read(Word),
    Write(Word, Word),
    Kill,
}

#[derive(Debug)]
pub struct MemoryController {
    tx: channel::Sender<MemoryOps>,
    rx: channel::Receiver<Option<Word>>,
}

impl MemoryController {
    pub fn read(&self, address: Word) -> Word {
        telemetry_log!(CONFIG.cycles.dram_read);
        self.tx
            .send(MemoryOps::Read(address))
            .expect("Panic in memory fabric");
        self.rx
            .recv()
            .expect("Panic in memory fabric")
            .expect("No response from memory fabric")
    }

    pub fn write(&mut self, address: Word, value: Word) {
        telemetry_log!(CONFIG.cycles.dram_write);
        self.tx
            .send(MemoryOps::Write(address, value))
            .expect("Panic in memory fabric");
        assert!(
            self.rx.recv().expect("Panic in memory fabric").is_none(),
            "Non empty response from memory fabric"
        );
    }

    pub fn kill(&mut self) {
        self.tx
            .send(MemoryOps::Kill)
            .expect("Panic in memory fabric");
        // We just interpret some kind of response as "terminating"
        let _ = self.rx.recv();
    }
}

#[derive(Debug)]
pub struct Dram {
    inner: Vec<u8>,
    tx: channel::Sender<Option<Word>>,
    rx: channel::Receiver<MemoryOps>,
    mirror: Option<channel::Sender<MemoryOps>>,
}

impl Dram {
    pub fn new() -> (Self, MemoryController) {
        telemetry_init!();
        let (op_tx, op_rx) = channel::unbounded();
        let (resp_tx, resp_rx) = channel::unbounded();
        let mc = MemoryController {
            tx: op_tx,
            rx: resp_rx,
        };
        (
            Self {
                inner: vec![0; Word::MAX as usize + 1],
                tx: resp_tx,
                rx: op_rx,
                mirror: None,
            },
            mc,
        )
    }

    pub fn mirror(&mut self) -> DramMirror {
        let (mirror_tx, mirror_rx) = crossbeam::channel::unbounded();
        self.mirror = Some(mirror_tx);
        DramMirror {
            inner: self.inner.clone(),
            rx: mirror_rx,
        }
    }

    fn read_byte(&self, addr: Word) -> u8 {
        self.inner[addr as usize]
    }

    fn read_cache_line(&self, addr: Word) -> CacheLine {
        let addr = cache_aligned!(addr);
        let mut line = 0;

        for i in 0..size_of::<CacheLine>() {
            line |= (self.read_byte(addr + i as u16) as u16) << (i * 8);
        }

        line
    }

    fn write_cache_line(&mut self, addr: Word, val: CacheLine) {
        for i in 0..size_of::<CacheLine>() {
            self.inner[addr as usize + i] = ((val >> (i * 8)) & 0xFF) as u8;
        }
    }
}

impl Block for Dram {
    fn dispatch(mut self) {
        loop {
            let op = self.rx.recv().expect("Panic in memory fabric");
            match op {
                MemoryOps::Read(addr) => {
                    self.tx
                        .send(Some(self.read_cache_line(addr)))
                        .expect("Panic in memory fabric");
                }
                MemoryOps::Write(addr, value) => {
                    self.write_cache_line(addr, value);
                    self.tx.send(None).expect("Panic in memory fabric");

                    // Replicate our write op to the mirror
                    if let Some(mirror) = self.mirror.as_ref() {
                        mirror.send(op).expect("Panic in memory mirror");
                    }
                }
                MemoryOps::Kill => return,
            }
        }
    }
}

pub struct DramMirror {
    inner: Vec<u8>,
    rx: channel::Receiver<MemoryOps>,
}

impl DramMirror {
    pub fn update(&mut self) {
        while let Ok(op) = self.rx.try_recv() {
            if let MemoryOps::Write(addr, value) = op {
                for i in 0..size_of::<CacheLine>() {
                    self.inner[addr as usize + i] = ((value >> (i * 8)) & 0xFF) as u8;
                }
            }
        }
    }
}

impl Widget for &DramMirror {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = ratatui::widgets::Block::bordered().title("DRAM");

        let inner = block.inner(area);
        block.render(area, buf);

        const BYTES_PER_ROW: usize = 16;

        for row in 0..inner.height as usize {
            let offset = row * BYTES_PER_ROW;

            if offset >= self.inner.len() {
                break;
            }

            let end = (offset + BYTES_PER_ROW).min(self.inner.len());
            let bytes = &self.inner[offset..end];

            let mut line = format!("{offset:04X}: ");

            for byte in bytes {
                line.push_str(&format!("{byte:02X} "));
            }

            buf.set_string(
                inner.x,
                inner.y + row as u16,
                line,
                Style::default().fg(Color::White),
            );
        }
    }
}
