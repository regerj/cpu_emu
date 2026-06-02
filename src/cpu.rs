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
}

impl Cpu {
    pub fn new(mc: MemoryController) -> Self {
        Self {
            regs: vec![0; 2],
            mc,
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
                    self.mc.read(src_word)
                } else {
                    src_word
                };

                match dest.ty {
                    OperandInner::Register => {
                        self.regs[dest.word as usize] += src_word;
                    }
                    // We asserted storable, so we know the literal is an address
                    OperandInner::Literal => {
                        let dest_val = self.mc.read(dest.word);
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
                    self.mc.read(src_word)
                } else {
                    src_word
                };

                match dest.ty {
                    OperandInner::Register => {
                        self.regs[dest.word as usize] -= src_word;
                    }
                    // We asserted storable, so we know the literal is an address
                    OperandInner::Literal => {
                        let dest_val = self.mc.read(dest.word);
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
                    self.mc.read(src_word)
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
}

impl Drop for Cpu {
    fn drop(&mut self) {
        self.mc.kill();
    }
}
