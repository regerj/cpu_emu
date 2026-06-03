telemetry_module!(cpu);

use crate::{
    WORD,
    cache::{
        Cache,
        CacheAddr,
    },
    mem::MemoryController,
    ops::{
        OperandInner,
        Operation,
    },
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

#[derive(Debug)]
pub struct Cpu {
    regs: Vec<WORD>,
    mc: MemoryController,
    cache: Cache,
}

impl Cpu {
    pub fn new(mc: MemoryController) -> Self {
        telemetry_init!();
        Self {
            regs: vec![0; 2],
            mc,
            cache: Cache::new(),
        }
    }

    pub fn execute(&mut self, op: Operation) {
        telemetry_log!(1);
        match op {
            Operation::Add(dest, src) => {
                assert!(dest.can_store());

                let src_word = match src.ty {
                    OperandInner::Register => self.regs[src.word as usize],
                    OperandInner::Literal => src.word,
                };

                let src_word = if src.deref {
                    self.read_addr(src_word)
                } else {
                    src_word
                };

                match dest.ty {
                    OperandInner::Register => {
                        self.regs[dest.word as usize] += src_word;
                    }
                    // We asserted storable, so we know the literal is an address
                    OperandInner::Literal => {
                        let dest_val = self.read_addr(dest.word);
                        self.mc.write(dest.word, dest_val + src_word);
                    }
                }
            }
            Operation::Sub(dest, src) => {
                assert!(dest.can_store());

                let src_word = match src.ty {
                    OperandInner::Register => self.regs[src.word as usize],
                    OperandInner::Literal => src.word,
                };

                let src_word = if src.deref {
                    self.read_addr(src_word)
                } else {
                    src_word
                };

                match dest.ty {
                    OperandInner::Register => {
                        self.regs[dest.word as usize] -= src_word;
                    }
                    // We asserted storable, so we know the literal is an address
                    OperandInner::Literal => {
                        let dest_val = self.read_addr(dest.word);
                        self.mc.write(dest.word, dest_val - src_word);
                    }
                }
            }
            Operation::Mov(dest, src) => {
                assert!(dest.can_store());

                let src_word = match src.ty {
                    OperandInner::Register => self.regs[src.word as usize],
                    OperandInner::Literal => src.word,
                };

                let src_word = if src.deref {
                    self.read_addr(src_word)
                } else {
                    src_word
                };

                match dest.ty {
                    OperandInner::Register => {
                        self.regs[dest.word as usize] = src_word;
                    }
                    // We asserted storable, so we know the literal is an address
                    OperandInner::Literal => {
                        self.mc.write(dest.word, src_word);
                    }
                }
            }
        }
    }

    /// Attempt to read the given address from the cache. Failing that, fallback to reading it from
    /// the memory controller.
    fn read_addr(&self, addr: WORD) -> WORD {
        let cache_addr = CacheAddr::from_bits(addr);
        if let Some(cache_entry) = self.cache.lookup(&cache_addr) {
            let val = (cache_entry >> (8 * cache_addr.offset()) & 0xFF) as u8;
            return val;
        }

        self.mc.read(addr)
    }
}

impl Drop for Cpu {
    fn drop(&mut self) {
        self.mc.kill();
    }
}
