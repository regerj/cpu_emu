use std::sync::mpsc;

use crate::{
    WORD,
    block::Block,
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(dram);

pub enum MemoryOps {
    Read(WORD),
    Write(WORD, WORD),
    Kill,
}

#[derive(Debug)]
pub struct MemoryController {
    tx: mpsc::Sender<MemoryOps>,
    rx: mpsc::Receiver<Option<WORD>>,
}

impl MemoryController {
    pub fn read(&self, address: WORD) -> WORD {
        telemetry_log!(300);
        self.tx
            .send(MemoryOps::Read(address))
            .expect("Panic in memory fabric");
        self.rx
            .recv()
            .expect("Panic in memory fabric")
            .expect("No response from memory fabric")
    }

    pub fn write(&mut self, address: WORD, value: WORD) {
        telemetry_log!(300);
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
    tx: mpsc::Sender<Option<WORD>>,
    rx: mpsc::Receiver<MemoryOps>,
}

impl Dram {
    pub fn new() -> (Self, MemoryController) {
        telemetry_init!();
        let (op_tx, op_rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();
        let mc = MemoryController {
            tx: op_tx,
            rx: resp_rx,
        };
        (
            Self {
                inner: vec![0; WORD::MAX as usize],
                tx: resp_tx,
                rx: op_rx,
            },
            mc,
        )
    }
}

impl Block for Dram {
    fn dispatch(mut self) {
        loop {
            let op = self.rx.recv().expect("Panic in memory fabric");
            match op {
                MemoryOps::Read(addr) => {
                    self.tx
                        .send(Some(self.inner[addr as usize]))
                        .expect("Panic in memory fabric");
                }
                MemoryOps::Write(addr, value) => {
                    self.inner[addr as usize] = value;
                    self.tx.send(None).expect("Panic in memory fabric")
                }
                MemoryOps::Kill => return,
            }
        }
    }
}
