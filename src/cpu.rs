use bitfield_struct::bitfield;

use crate::{
    WORD,
    mem::MemoryController,
    ops::{
        OperandInner,
        Operation,
    },
};

#[derive(Debug)]
pub struct Cpu {
    regs: Vec<WORD>,
    mc: MemoryController,
    cache: Cache,
}

impl Cpu {
    pub fn new(mc: MemoryController) -> Self {
        Self {
            regs: vec![0; 2],
            mc,
            cache: Cache::new(),
        }
    }

    pub fn execute(&mut self, op: Operation) {
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

/// The cache is a 1 level 2-way set associative cache.
///
/// It utilizes 2 byte cache line size and has a total capacity of 16 cache lines.
#[derive(Debug)]
struct Cache {
    inner: [[Option<CacheEntry>; 2]; 8],
}

impl Cache {
    fn new() -> Self {
        Self {
            inner: [[None; 2]; 8],
        }
    }

    fn lookup(&self, addr: &CacheAddr) -> Option<u16> {
        self.inner[addr.index()].iter().find_map(|entry| {
            let entry = (*entry)?;
            if entry.tag == addr.tag() {
                Some(entry.val)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct CacheEntry {
    tag: u8,
    val: u16,
}

#[bitfield(u8)]
struct CacheAddr {
    #[bits(1)]
    offset: usize,
    #[bits(3)]
    index: usize,
    #[bits(4)]
    tag: u8,
}
