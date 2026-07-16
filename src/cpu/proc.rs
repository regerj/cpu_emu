telemetry_module!(cpu);

use common::{
    cfg::{
        CacheLine,
        Word,
    },
    isa::{
        Operand,
        OperandInner,
        Operation,
        Register,
    },
};
use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint,
        Layout,
        Rect,
    },
    style::Style,
    text::Line,
    widgets::{
        Block,
        Paragraph,
        Widget,
    },
};

use crate::{
    cache_aligned,
    cpu::{
        cache::{
            Cache,
            CacheAddr,
        },
        regs::{
            Regs,
            StatusRegister,
        },
    },
    is_cache_aligned,
    mem::{
        MemoryController,
        Offset,
        PhysAddr,
    },
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

/// A type that will iterate the physical address space for the given CPU.
///
/// It's iteration will begin at the current value of $IP and will increment $IP as it consumes
/// bytes.
///
/// # Safety
/// Because this iterator directly modifies $IP byte by byte, improper use can cause $IP to point to
/// something other than an intended mneumonic byte. It mainly exists as a facilitator type for the
/// `retrieve_next_instruction()` method on the `Cpu` directly. It generally should not be used
/// directly.
struct IpIter<'a> {
    cpu: &'a mut Cpu,
}

impl Iterator for IpIter<'_> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let byte = self
            .cpu
            .read_byte(PhysAddr::new(self.cpu.regs[Register::IP]));
        self.cpu.regs[Register::IP] += 1;
        Some(byte)
    }
}

struct InertIpIter<'a> {
    cpu: &'a Cpu,
    ip: PhysAddr,
}

impl<'a> InertIpIter<'a> {
    fn new(cpu: &'a Cpu) -> Self {
        Self {
            cpu,
            ip: PhysAddr::new(cpu.regs[Register::IP]),
        }
    }
}

impl Iterator for InertIpIter<'_> {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        let byte = self.cpu.sideband_read(self.ip);
        self.ip += Offset::new(1);
        Some(byte)
    }
}

#[derive(Debug)]
pub struct Cpu {
    pub regs: Regs,
    mc: MemoryController,
    pub cache: Cache,
}

impl Widget for &Cpu {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let chunks =
            Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
        let (upper_chunk, lower_chunk) = (chunks[0], chunks[1]);

        let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(upper_chunk);
        let (left_chunk, right_chunk) = (chunks[0], chunks[1]);

        let mut ip_iter = InertIpIter::new(self);
        let mut instructions = vec![];
        let mut i = 0;
        while let Ok(Some(op)) = Operation::consume(&mut ip_iter)
            && i < 5
        {
            let style = if i == 0 {
                Style::default().bold()
            } else {
                Style::default().dim()
            };
            instructions.push(Line::from(op.to_string()).style(style));
            i += 1;
        }

        Paragraph::new(instructions)
            .block(Block::bordered().title("Instructions"))
            .render(left_chunk, buf);
        self.regs.render(right_chunk, buf);
        self.cache.render(lower_chunk, buf);
    }
}

impl<'a> Cpu {
    pub fn new(mc: MemoryController) -> Self {
        telemetry_init!();
        Self {
            regs: Regs::new(),
            mc,
            cache: Cache::new(),
        }
    }

    fn iter_mem(&'a mut self) -> IpIter<'a> {
        IpIter { cpu: self }
    }

    pub fn execute(&mut self) -> Option<()> {
        telemetry_log!(1);

        let instruction = self.retrieve_next_instruction();
        match instruction {
            Operation::Add(dest, src) => {
                let src_word = match src {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        self.read_word(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(reg) => self.regs[reg],
                    },
                };

                match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        let dest_word = self.read_word(addr);

                        self.write_addr(addr, dest_word + src_word);
                    }
                    Operand::RValue(OperandInner::Register(reg)) => {
                        self.regs[reg] += src_word;
                    }
                    Operand::RValue(OperandInner::Literal(..)) => {
                        panic!("RValue literals are not allowed as destinations")
                    }
                }
            }
            Operation::Sub(dest, src) => {
                let src_word = match src {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        self.read_word(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(reg) => self.regs[reg],
                    },
                };

                match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        let dest_word = self.read_word(addr);

                        self.write_addr(addr, dest_word - src_word);
                    }
                    Operand::RValue(OperandInner::Register(reg)) => {
                        self.regs[reg] -= src_word;
                    }
                    Operand::RValue(OperandInner::Literal(..)) => {
                        panic!("RValue literals are not allowed as destinations")
                    }
                }
            }
            Operation::Mov(dest, src) => {
                let src_word = match src {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        self.read_word(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(reg) => self.regs[reg],
                    },
                };

                match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        self.write_addr(addr, src_word);
                    }
                    Operand::RValue(OperandInner::Register(reg)) => {
                        self.regs[reg] = src_word;
                    }
                    Operand::RValue(OperandInner::Literal(..)) => {
                        panic!("RValue literals are not allowed as destinations")
                    }
                }
            }
            Operation::Jmp(dest) => {
                let addr: PhysAddr = match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());
                        PhysAddr::new(self.read_word(addr))
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => PhysAddr::new(word),
                        OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                    },
                };

                self.regs[Register::IP] = addr.into_raw();
            }
            Operation::Cmp(op0, op1) => {
                let op0_word = match op0 {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        self.read_word(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(reg) => self.regs[reg],
                    },
                };

                let op1_word = match op1 {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => PhysAddr::new(word),
                            OperandInner::Register(reg) => PhysAddr::new(self.regs[reg]),
                        };

                        assert!(addr.is_word_aligned());

                        self.read_word(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(reg) => self.regs[reg],
                    },
                };

                let mut status = self.status();
                status.set_zero(op0_word == op1_word);
                self.set_status(&status);
            }
        }

        Some(())
    }

    /// Get the current value of the status register.
    fn status(&self) -> StatusRegister {
        StatusRegister::from_bits(self.regs[Register::ST])
    }

    /// Set the value of the status register.
    fn set_status(&mut self, v: &StatusRegister) {
        self.regs[Register::ST] = v.into_bits();
    }

    /// Read the next instruction from the address located in $IP.
    ///
    /// Increment the value of $IP by the appropriate number of bytes to point to the instruction
    /// following the returned instruction.
    fn retrieve_next_instruction(&mut self) -> Operation {
        // We reinstantiate this iter each time in case instruction modified $IP (ex: jmp)
        let mut iter = self.iter_mem();
        Operation::consume(&mut iter)
            .expect("Invalid machine code")
            .expect("Unexpected end of instruction queue.")
    }

    /// Attempt to read the given address from the cache. Failing that, fallback to reading it from
    /// the memory controller.
    ///
    /// Address need not be aligned.
    fn read_byte(&mut self, addr: PhysAddr) -> u8 {
        let cache_addr: CacheAddr = addr.into();
        let cache_line = self.read_cache_line(addr);

        (cache_line >> (cache_addr.offset() * 8)) as u8
    }

    /// Attempt to read the given address from the cache. Failing that, fallback to reading it from
    /// the memory controller.
    ///
    /// Address must be word aligned.
    fn read_word(&mut self, addr: PhysAddr) -> Word {
        assert!(addr.is_word_aligned());
        let cache_addr: CacheAddr = addr.into();
        let cache_line = self.read_cache_line(addr);

        (cache_line >> (cache_addr.offset() * 8)) as Word
    }

    /// Read a cache line from the cache, or failing that, read from main memory and populate the
    /// cache.
    ///
    /// Address does not need to be cache aligned, but the returned `CacheLine` will be.
    fn read_cache_line(&mut self, addr: impl Into<CacheAddr>) -> CacheLine {
        let cache_addr = addr.into();
        if let Some(cache_entry) = self.cache.lookup(cache_addr) {
            return cache_entry;
        }

        let cache_line =
            self.read_cache_line_from_mem(PhysAddr::new(cache_aligned!(cache_addr.into_bits())));
        // Our cpu can use cache line without rereading
        self.cache.insert(cache_addr, cache_line);

        cache_line
    }

    /// Write a value to an address.
    ///
    /// Value *must* be able to fit within a single cache line.
    ///
    /// Address is not required to be cache aligned, but is required to be word aligned.
    fn write_addr(&mut self, addr: PhysAddr, value: Word) {
        assert!(addr.is_word_aligned());
        let cache_addr: CacheAddr = addr.into();
        let mut cache_line = self.read_cache_line(addr);
        let zero_mask: CacheLine = !((Word::MAX as CacheLine) << (cache_addr.offset() * 8));
        cache_line &= zero_mask;

        cache_line |= (value as CacheLine) << (cache_addr.offset() * 8);

        self.cache.insert(addr, cache_line);
        for i in 0..size_of::<Word>() {
            let byte: u8 = (value >> (8 * i)) as _;
            self.mc.write(addr + Offset::new(i as u16), byte);
        }
    }

    /// Read an entire cache line from main memory.
    ///
    /// Address must be cache aligned.
    fn read_cache_line_from_mem(&self, addr: PhysAddr) -> CacheLine {
        assert!(is_cache_aligned!(addr.into_raw()));
        let mut cache_line: CacheLine = 0;
        for i in 0..size_of::<CacheLine>() {
            cache_line |= (self.mc.read(addr + Offset::new(i as Word)) as CacheLine) << (8 * i);
        }

        cache_line
    }

    /// Read a byte from the given address without affecting the state of the CPU.
    fn sideband_read(&self, addr: PhysAddr) -> u8 {
        self.mc.read(addr)
    }
}

impl Drop for Cpu {
    fn drop(&mut self) {
        self.mc.kill();
    }
}

#[cfg(test)]
mod tests {
    use common::isa::{
        Operand,
        OperandInner,
        Operation,
        Register,
    };

    use crate::{
        block::Block,
        cpu::Cpu,
        mem::{
            Dram,
            MemoryController,
            PhysAddr,
        },
    };

    type CpuValidationFn = Box<dyn FnOnce(&mut Cpu)>;
    type DramValidationFn = Box<dyn FnOnce(&mut MemoryController)>;
    struct CpuTester {
        op: Operation,
        pre_validation: Option<CpuValidationFn>,
        post_validation: Option<CpuValidationFn>,
        dram_seeder: Option<DramValidationFn>,
    }

    impl CpuTester {
        fn new(op: Operation) -> Self {
            Self {
                op,
                pre_validation: None,
                post_validation: None,
                dram_seeder: None,
            }
        }

        fn with_pre_validation(mut self, f: impl FnOnce(&mut Cpu) + 'static) -> Self {
            self.pre_validation = Some(Box::new(f));
            self
        }

        fn with_post_validation(mut self, f: impl FnOnce(&mut Cpu) + 'static) -> Self {
            self.post_validation = Some(Box::new(f));
            self
        }

        fn with_dram_seeder(mut self, f: impl FnOnce(&mut MemoryController) + 'static) -> Self {
            self.dram_seeder = Some(Box::new(f));
            self
        }

        fn test(self) {
            let (dram, dram_radio) = Dram::new();
            let mut mem_ctrl = MemoryController::new();
            mem_ctrl.reg_mem_ep(dram_radio);

            let dram_handle = std::thread::spawn(move || dram.dispatch());

            if let Some(seeder) = self.dram_seeder {
                seeder(&mut mem_ctrl);
            }

            // Test adding lit to register
            let mut cpu = Cpu::new(mem_ctrl);

            if let Some(pre_validation) = self.pre_validation {
                pre_validation(&mut cpu);
            }

            while cpu.execute().is_some() {}

            if let Some(post_validation) = self.post_validation {
                post_validation(&mut cpu);
            }

            drop(cpu);

            dram_handle.join().unwrap();
        }
    }

    fn r(n: &str) -> Operand {
        Operand::RValue(OperandInner::Register(
            Register::try_from(n).expect("Bad register name"),
        ))
    }

    fn r_star(n: &str) -> Operand {
        Operand::LValue(OperandInner::Register(
            Register::try_from(n).expect("Bad register name"),
        ))
    }

    fn lit(n: u16) -> Operand {
        Operand::RValue(OperandInner::Literal(n))
    }

    fn lit_star(n: u16) -> Operand {
        Operand::LValue(OperandInner::Literal(n))
    }

    fn add(op0: Operand, op1: Operand) -> Operation {
        Operation::Add(op0, op1)
    }

    #[test]
    fn test_add() {
        // Test adding lit to reg
        CpuTester::new(add(r("r0"), lit(1)))
            .with_post_validation(|cpu| assert_eq!(cpu.regs[Register::R0], 1))
            .test();
        // Test adding reg to reg
        CpuTester::new(add(r("r0"), r("r1")))
            .with_pre_validation(|cpu| {
                cpu.regs[Register::R0] = 2;
                cpu.regs[Register::R1] = 3;
            })
            .with_post_validation(|cpu| assert_eq!(cpu.regs[Register::R0], 5))
            .test();

        // Test adding lit to *reg
        CpuTester::new(add(r_star("r0"), lit(0xFF00))).with_post_validation(|cpu| {
            let entry = cpu.cache.lookup(0).expect("Cache entry not present");
            assert_eq!(entry, 0xFF00);

            assert_eq!(cpu.mc.read(PhysAddr::new(0)), 0);
            assert_eq!(cpu.mc.read(PhysAddr::new(1)), 0xFF);
        });

        // Test adding reg to *reg
        CpuTester::new(add(r_star("r0"), r("r1")))
            .with_dram_seeder(|mc| mc.write(PhysAddr::new(0x00), 0xAD))
            .with_pre_validation(|cpu| {
                cpu.regs[Register::R1] = 0xDE00;
            })
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(PhysAddr::new(0)), 0xAD);
                assert_eq!(cpu.mc.read(PhysAddr::new(1)), 0xDE);
            });

        // Test adding lit to *lit
        CpuTester::new(add(lit_star(0x00), lit(0xDE00)))
            .with_dram_seeder(|mc| mc.write(PhysAddr::new(0x00), 0xAD))
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(PhysAddr::new(0)), 0xAD);
                assert_eq!(cpu.mc.read(PhysAddr::new(1)), 0xDE);
            });

        // Test adding reg to *lit
        CpuTester::new(add(lit_star(0x00), r("r0")))
            .with_dram_seeder(|mc| mc.write(PhysAddr::new(0x00), 0xAD))
            .with_pre_validation(|cpu| {
                cpu.regs[Register::R0] = 0xDE00;
            })
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(PhysAddr::new(0)), 0xAD);
                assert_eq!(cpu.mc.read(PhysAddr::new(1)), 0xDE);
            });

        // Test adding *reg to *reg
        CpuTester::new(add(r_star("r0"), r_star("r1")))
            .with_dram_seeder(|mc| {
                mc.write(PhysAddr::new(0x00), 0xAD);
                mc.write(PhysAddr::new(0x03), 0xDE);
            })
            .with_pre_validation(|cpu| {
                cpu.regs[Register::R0] = 0x00;
                cpu.regs[Register::R1] = 0x02;
            })
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(PhysAddr::new(0)), 0xAD);
                assert_eq!(cpu.mc.read(PhysAddr::new(1)), 0xDE);
            });
    }
}
