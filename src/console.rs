use std::{
    sync::{
        Arc,
        RwLock,
    },
    thread::JoinHandle,
};

use common::cfg::Word;
use crossbeam::channel::{
    Receiver,
    Sender,
};
use ratatui::widgets::{
    Block,
    Paragraph,
    Widget,
};

use crate::{
    block::Handle,
    mem::{
        MemoryFabricEndpoint,
        MemoryOps,
        Offset,
        PhysAddr,
    },
};

const BUFFER_LEN: usize = 64;

pub struct ConsoleEP {
    tx: Sender<MemoryOps>,
    rx: Receiver<Option<u8>>,
}

impl ConsoleEP {
    const REGION: crate::mem::MemoryRegion =
        crate::mem::MemoryRegion::new(PhysAddr::new(0xE000), Offset::new(BUFFER_LEN as Word));

    fn normalize_addr(addr: PhysAddr) -> Offset {
        addr - Self::REGION.begin
    }
}

impl MemoryFabricEndpoint for ConsoleEP {
    fn id(&self) -> Option<String> {
        Some("Console".to_string())
    }

    fn region(&self) -> crate::mem::MemoryRegion {
        Self::REGION
    }

    fn read_byte(&self, addr: PhysAddr) -> u8 {
        self.tx
            .send(MemoryOps::Read(Self::normalize_addr(addr)))
            .expect("Panic in memory fabric");
        self.rx
            .recv()
            .expect("Panic in memory fabric")
            .expect("Epxected non-empty response to read operation")
    }

    fn write_byte(&mut self, addr: PhysAddr, val: u8) {
        self.tx
            .send(MemoryOps::Write(Self::normalize_addr(addr), val))
            .expect("Panic in memory fabric");
        assert!(self.rx.recv().expect("Panic in memory fabric").is_none());
    }

    fn kill(&self) {
        self.tx
            .send(MemoryOps::Kill)
            .expect("Panic in memory fabric");
        assert!(self.rx.recv().expect("Panic in memory fabric").is_none());
    }
}

pub struct Console {
    buffer: Arc<RwLock<[u8; BUFFER_LEN]>>,
    rx: Receiver<MemoryOps>,
    tx: Sender<Option<u8>>,
}

impl Console {
    pub fn new() -> (Self, ConsoleEP) {
        let (cmd_tx, cmd_rx) = crossbeam::channel::unbounded();
        let (resp_tx, resp_rx) = crossbeam::channel::unbounded();

        let ep = ConsoleEP {
            tx: cmd_tx,
            rx: resp_rx,
        };

        let slf = Self {
            buffer: Arc::new(RwLock::new([0; BUFFER_LEN])),
            tx: resp_tx,
            rx: cmd_rx,
        };

        (slf, ep)
    }
}

impl crate::block::Block<ConsoleHandle> for Console {
    fn dispatch(self) -> ConsoleHandle {
        let buffer = self.buffer.clone();

        let handle = std::thread::spawn(move || {
            loop {
                match self.rx.recv().expect("Panic in memory fabric") {
                    MemoryOps::Read(addr) => {
                        let buf_lock = self.buffer.read().expect("Panic in console buffer lock");
                        self.tx
                            .send(Some(buf_lock[addr.into_raw() as usize]))
                            .expect("Panic in memory fabric");
                    }
                    MemoryOps::Write(addr, val) => {
                        let mut buf_lock =
                            self.buffer.write().expect("Panic in console buffer lock");
                        buf_lock[addr.into_raw() as usize] = val;
                        self.tx.send(None).expect("Panic in memory fabric");
                    }
                    MemoryOps::Kill => {
                        self.tx.send(None).expect("Panic in memory fabric");
                        return;
                    }
                }
            }
        });

        ConsoleHandle { buffer, handle }
    }
}

pub struct ConsoleHandle {
    buffer: Arc<RwLock<[u8; BUFFER_LEN]>>,
    handle: JoinHandle<()>,
}

impl Handle for ConsoleHandle {
    fn get_widget(&self) -> impl Widget {
        self
    }
}

impl Widget for &ConsoleHandle {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().title("Console");
        let buffer_lock = self.buffer.read().expect("Panic on Console lock");
        // let s: String = buffer_lock.iter().map(|byte| {
        //     byte.to_string()
        // }).collect();
        let s: String = buffer_lock.iter().map(|byte| *byte as char).collect();
        Paragraph::new(s).block(block).render(area, buf);
    }
}
