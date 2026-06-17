telemetry_module!(cpu);

use crate::{
    WORD,
    aligned,
    cache::{
        Cache,
        CacheAddr,
        CacheLine,
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
    pub cache: Cache,
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
                        self.write_addr(dest.word, dest_val + src_word);
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
    fn read_addr(&mut self, addr: WORD) -> WORD {
        let cache_addr: CacheAddr = addr.into();
        let cache_line = self.read_cache_line(addr);

        (cache_line >> (cache_addr.offset() * 8)) as WORD
    }

    fn read_cache_line(&mut self, addr: impl Into<CacheAddr>) -> CacheLine {
        let cache_addr = addr.into();
        if let Some(cache_entry) = self.cache.lookup(cache_addr) {
            return cache_entry;
        }

        let cache_line = self.read_cache_line_from_mem(cache_addr.into_bits());
        // Our cpu can use cache line without rereading
        self.cache.insert(cache_addr, cache_line);

        cache_line
    }

    fn write_addr(&mut self, addr: WORD, value: WORD) {
        let cache_addr: CacheAddr = addr.into();
        let mut cache_line = self.read_cache_line(addr);
        let zero_mask: CacheLine = !((WORD::MAX as CacheLine) << (cache_addr.offset() * 8));
        cache_line &= zero_mask;

        cache_line |= (value as CacheLine) << (cache_addr.offset() * 8);
        self.cache.insert(addr, cache_line);
    }

    /// Read an entire cache line from main memory
    fn read_cache_line_from_mem(&self, addr: WORD) -> CacheLine {
        let mut cache_line: CacheLine = 0;

        for i in 0..std::mem::size_of::<CacheLine>() {
            let byte = self.mc.read(aligned!(addr) + i as WORD);
            cache_line |= (byte as CacheLine) << (8 * i);
        }

        cache_line
    }
}

impl Drop for Cpu {
    fn drop(&mut self) {
        self.mc.kill();
    }
}
