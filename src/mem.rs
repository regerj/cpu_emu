use crossbeam::channel;
use log::debug;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{
        Line,
        Span,
        Text,
    },
    widgets::{
        Paragraph,
        Widget,
    },
};

use crate::{
    block::Block,
    cfg::{
        CHANGE_STYLE,
        CONFIG,
        Word,
    },
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(dram);

pub enum MemoryOps {
    Read(Word),
    Write(Word, u8),
    Kill,
}

#[derive(Debug)]
pub struct MemoryController {
    tx: channel::Sender<MemoryOps>,
    rx: channel::Receiver<Option<u8>>,
}

impl MemoryController {
    /// Read a byte from main memory.
    ///
    /// Address does not need to be aligned.
    pub fn read(&self, address: Word) -> u8 {
        telemetry_log!(CONFIG.cycles.dram_read);
        self.tx
            .send(MemoryOps::Read(address))
            .expect("Panic in memory fabric");
        self.rx
            .recv()
            .expect("Panic in memory fabric")
            .expect("No response from memory fabric")
    }

    /// Write a byte to main memory.
    ///
    /// Address does not need to be aligned.
    pub fn write(&self, address: Word, value: u8) {
        telemetry_log!(CONFIG.cycles.dram_write);
        self.tx
            .send(MemoryOps::Write(address, value))
            .expect("Panic in memory fabric");
        assert!(
            self.rx.recv().expect("Panic in memory fabric").is_none(),
            "Non empty response from memory fabric"
        );
    }

    pub fn kill(&self) {
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
    tx: channel::Sender<Option<u8>>,
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
            highlights: Vec::new(),
        }
    }

    fn read_byte(&self, addr: Word) -> u8 {
        self.inner[addr as usize]
    }

    fn write_byte(&mut self, addr: Word, value: u8) {
        self.inner[addr as usize] = value;
    }
}

impl Block for Dram {
    fn dispatch(mut self) {
        loop {
            let op = self.rx.recv().expect("Panic in memory fabric");
            match op {
                MemoryOps::Read(addr) => {
                    debug!("DRAM reading address 0x{addr:04X}");
                    self.tx
                        .send(Some(self.read_byte(addr)))
                        .expect("Panic in memory fabric");
                }
                MemoryOps::Write(addr, value) => {
                    debug!("DRAM writing address 0x{addr:04x} with value 0x{value:04x}");
                    self.write_byte(addr, value);
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
    highlights: Vec<usize>,
    rx: channel::Receiver<MemoryOps>,
}

impl DramMirror {
    pub fn update(&mut self) {
        while let Ok(op) = self.rx.try_recv() {
            if let MemoryOps::Write(addr, value) = op {
                self.inner[addr as usize] = value;
                self.highlights.push(addr as usize);
            }
        }
    }

    pub fn clear_highlights(&mut self) {
        self.highlights.clear();
    }
}

impl Widget for &DramMirror {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const BYTES_PER_ROW: usize = 16;
        // TODO this some absolute slop, idk why its writing direct to buffer...
        // rewrite it as a text containing lines containing spans for each byte like a well adjusted
        // software engineer not some cretin
        let block = ratatui::widgets::Block::bordered().title("DRAM");
        let rows = self.inner.chunks(BYTES_PER_ROW);
        let mut text = Text::default();
        for (i, row) in rows.enumerate() {
            let offset = i * BYTES_PER_ROW;
            let mut line = Line::from(format!("{offset:04X}: "));

            for (j, byte) in row.iter().enumerate() {
                let span = if self.highlights.contains(&(offset + j)) {
                    Span::from(format!("{byte:02X}")).style(*CHANGE_STYLE)
                } else {
                    Span::from(format!("{byte:02X}"))
                };
                line.push_span(span);
                line.push_span(Span::from(" "));
            }

            text.push_line(line);
        }

        Paragraph::new(text).block(block).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::Block,
        mem::Dram,
    };

    #[test]
    fn test_read() {
        let (mut dram, mc) = Dram::new();
        dram.inner[0] = 0xDE;
        dram.inner[1] = 0xAD;
        dram.inner[2] = 0xBE;
        dram.inner[3] = 0xEF;

        let dram_handle = std::thread::spawn(move || dram.dispatch());

        assert_eq!(mc.read(0x0), 0xDE);
        assert_eq!(mc.read(0x1), 0xAD);
        assert_eq!(mc.read(0x2), 0xBE);
        assert_eq!(mc.read(0x3), 0xEF);

        mc.kill();
        dram_handle.join().unwrap();
    }

    #[test]
    fn test_write() {
        let (dram, mc) = Dram::new();
        let dram_handle = std::thread::spawn(move || dram.dispatch());

        mc.write(0x0, 0xDE);
        mc.write(0x1, 0xAD);
        mc.write(0x2, 0xBE);
        mc.write(0x3, 0xEF);

        assert_eq!(mc.read(0x0), 0xDE);
        assert_eq!(mc.read(0x1), 0xAD);
        assert_eq!(mc.read(0x2), 0xBE);
        assert_eq!(mc.read(0x3), 0xEF);

        mc.kill();
        dram_handle.join().unwrap();
    }
}
