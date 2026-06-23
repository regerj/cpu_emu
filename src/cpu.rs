telemetry_module!(cpu);

use std::{
    collections::HashMap,
    ops::{
        Index,
        IndexMut,
    },
    vec::IntoIter,
};

use crate::{
    cache::{
        Cache,
        CacheAddr,
    },
    cache_aligned,
    cfg::{
        CHANGE_STYLE,
        CacheLine,
        Word,
    },
    is_cache_aligned,
    is_word_aligned,
    mem::MemoryController,
    ops::{
        Operand,
        OperandInner,
        Operation,
    },
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

#[derive(Debug)]
pub struct Cpu {
    pub regs: Regs,
    mc: MemoryController,
    pub cache: Cache,
    instructions: IntoIter<crate::ops::Operation>,
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

        let mut iter = self.instructions.as_ref().iter();
        let mut instructions = vec![];
        if let Some(instruction) = iter.next() {
            instructions.push(Line::from(instruction.to_string()).style(Style::default().bold()));
        }

        iter.for_each(|elem| {
            instructions.push(Line::from(elem.to_string()).style(Style::default().dim()))
        });

        Paragraph::new(instructions)
            .block(Block::bordered().title("Instructions"))
            .render(left_chunk, buf);
        self.regs.render(right_chunk, buf);
        self.cache.render(lower_chunk, buf);
    }
}

#[derive(Debug)]
pub struct Regs {
    inner: HashMap<String, Reg>,
}

impl Regs {
    fn new() -> Self {
        Self {
            inner: HashMap::from([
                ("r0".to_string(), Reg::new()),
                ("r1".to_string(), Reg::new()),
                ("r2".to_string(), Reg::new()),
                ("r3".to_string(), Reg::new()),
            ]),
        }
    }

    pub fn clear_highlights(&mut self) {
        self.inner
            .values_mut()
            .for_each(|reg| reg.highlighted = false);
    }
}

#[derive(Debug)]
pub struct Reg {
    val: Word,
    highlighted: bool,
}

impl Reg {
    fn new() -> Self {
        Self {
            val: 0,
            highlighted: false,
        }
    }
}

impl From<&Reg> for Span<'_> {
    fn from(value: &Reg) -> Self {
        // let name = Span::from(format!("r{}", value.name));
        // let spacer = Span::from("  │ ");
        if value.highlighted {
            Span::from(format!("0x{:04X}", value.val)).style(*CHANGE_STYLE)
        } else {
            Span::from(format!("0x{:04X}", value.val))
        }
    }
}

use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint,
        Layout,
        Rect,
    },
    style::{
        Modifier,
        Style,
    },
    text::{
        Line,
        Span,
    },
    widgets::{
        Block,
        Paragraph,
        Widget,
    },
};

impl Widget for &Regs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Reg │ Value",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("────┼────────"),
        ];

        for (name, reg) in self.inner.iter() {
            let name = Span::from(format!("{}", name));
            let spacer = Span::from("  │ ");
            let reg: Span = reg.into();
            lines.push(Line::from(vec![name, spacer, reg]));
        }

        Paragraph::new(lines)
            .block(Block::bordered().title("Registers"))
            .render(area, buf);
    }
}

impl Index<&str> for Regs {
    type Output = Word;
    fn index(&self, index: &str) -> &Self::Output {
        &self.inner.get(index).expect("No register by that name").val
    }
}

impl IndexMut<&str> for Regs {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        let reg = self.inner.get_mut(index).expect("No register by that name");
        reg.highlighted = true;
        &mut reg.val
    }
}

impl Cpu {
    pub fn new(mc: MemoryController, instructions: IntoIter<crate::ops::Operation>) -> Self {
        telemetry_init!();
        Self {
            regs: Regs::new(),
            mc,
            cache: Cache::new(),
            instructions,
        }
    }

    pub fn execute(&mut self) -> Option<()> {
        telemetry_log!(1);

        let instruction = self.instructions.next()?;
        match instruction {
            Operation::Add(dest, src) => {
                let src_word = match src {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => word,
                            OperandInner::Register(name) => self.regs[&name],
                        };

                        assert!(is_word_aligned!(addr));

                        self.read_addr(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(name) => self.regs[&name],
                    },
                };

                match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => word,
                            OperandInner::Register(name) => self.regs[&name],
                        };

                        assert!(is_word_aligned!(addr));

                        let dest_word = self.read_addr(addr);

                        self.write_addr(addr, dest_word + src_word);
                    }
                    Operand::RValue(OperandInner::Register(name)) => {
                        self.regs[&name] += src_word;
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
                            OperandInner::Literal(word) => word,
                            OperandInner::Register(name) => self.regs[&name],
                        };

                        assert!(is_word_aligned!(addr));

                        self.read_addr(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(name) => self.regs[&name],
                    },
                };

                match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => word,
                            OperandInner::Register(name) => self.regs[&name],
                        };

                        assert!(is_word_aligned!(addr));

                        let dest_word = self.read_addr(addr);

                        self.write_addr(addr, dest_word - src_word);
                    }
                    Operand::RValue(OperandInner::Register(reg)) => {
                        self.regs[&reg] -= src_word;
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
                            OperandInner::Literal(word) => word,
                            OperandInner::Register(name) => self.regs[&name],
                        };

                        assert!(is_word_aligned!(addr));

                        self.read_addr(addr)
                    }
                    Operand::RValue(inner) => match inner {
                        OperandInner::Literal(word) => word,
                        OperandInner::Register(name) => self.regs[&name],
                    },
                };

                match dest {
                    Operand::LValue(inner) => {
                        let addr = match inner {
                            OperandInner::Literal(word) => word,
                            OperandInner::Register(name) => self.regs[&name],
                        };

                        assert!(is_word_aligned!(addr));

                        self.write_addr(addr, src_word);
                    }
                    Operand::RValue(OperandInner::Register(name)) => {
                        self.regs[&name] = src_word;
                    }
                    Operand::RValue(OperandInner::Literal(..)) => {
                        panic!("RValue literals are not allowed as destinations")
                    }
                }
            }
        }

        Some(())
    }

    /// Attempt to read the given address from the cache. Failing that, fallback to reading it from
    /// the memory controller.
    ///
    /// Address must be word aligned.
    fn read_addr(&mut self, addr: Word) -> Word {
        assert!(is_word_aligned!(addr));
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

        let cache_line = self.read_cache_line_from_mem(cache_aligned!(cache_addr.into_bits()));
        // Our cpu can use cache line without rereading
        self.cache.insert(cache_addr, cache_line);

        cache_line
    }

    /// Write a value to an address.
    ///
    /// Value *must* be able to fit within a single cache line.
    ///
    /// Address is not required to be cache aligned, but is required to be word aligned.
    fn write_addr(&mut self, addr: Word, value: Word) {
        assert!(is_word_aligned!(addr));
        let cache_addr: CacheAddr = addr.into();
        let mut cache_line = self.read_cache_line(addr);
        let zero_mask: CacheLine = !((Word::MAX as CacheLine) << (cache_addr.offset() * 8));
        cache_line &= zero_mask;

        cache_line |= (value as CacheLine) << (cache_addr.offset() * 8);

        self.cache.insert(addr, cache_line);
        for i in 0..size_of::<Word>() {
            let byte: u8 = (value >> (8 * i)) as _;
            self.mc.write(addr + i as Word, byte);
        }
    }

    /// Read an entire cache line from main memory.
    ///
    /// Address must be cache aligned.
    fn read_cache_line_from_mem(&self, addr: Word) -> CacheLine {
        assert!(is_cache_aligned!(addr));
        let mut cache_line: CacheLine = 0;
        for i in 0..size_of::<CacheLine>() {
            cache_line |= (self.mc.read(addr + i as Word) as CacheLine) << (8 * i);
        }

        cache_line
    }
}

impl Drop for Cpu {
    fn drop(&mut self) {
        self.mc.kill();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::Block,
        cpu::Cpu,
        mem::{
            Dram,
            MemoryController,
        },
        ops::{
            Operand,
            OperandInner,
            Operation,
        },
    };

    type CpuValidationFn = Box<dyn FnOnce(&mut Cpu)>;
    type DramValidationFn = Box<dyn FnOnce(&MemoryController)>;
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

        fn with_dram_seeder(mut self, f: impl FnOnce(&MemoryController) + 'static) -> Self {
            self.dram_seeder = Some(Box::new(f));
            self
        }

        fn test(self) {
            let (dram, mc) = Dram::new();

            let dram_handle = std::thread::spawn(move || dram.dispatch());

            if let Some(seeder) = self.dram_seeder {
                seeder(&mc);
            }

            // Test adding lit to register
            let mut cpu = Cpu::new(mc, vec![self.op].into_iter());

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
        Operand::RValue(OperandInner::Register(n.to_string()))
    }

    fn r_star(n: &str) -> Operand {
        Operand::LValue(OperandInner::Register(n.to_string()))
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
            .with_post_validation(|cpu| assert_eq!(cpu.regs["r0"], 1))
            .test();
        // Test adding reg to reg
        CpuTester::new(add(r("r0"), r("r1")))
            .with_pre_validation(|cpu| {
                cpu.regs["r0"] = 2;
                cpu.regs["r1"] = 3;
            })
            .with_post_validation(|cpu| assert_eq!(cpu.regs["r0"], 5))
            .test();

        // Test adding lit to *reg
        CpuTester::new(add(r_star("r0"), lit(0xFF00))).with_post_validation(|cpu| {
            let entry = cpu.cache.lookup(0).expect("Cache entry not present");
            assert_eq!(entry, 0xFF00);

            assert_eq!(cpu.mc.read(0), 0);
            assert_eq!(cpu.mc.read(1), 0xFF);
        });

        // Test adding reg to *reg
        CpuTester::new(add(r_star("r0"), r("r1")))
            .with_dram_seeder(|mc| mc.write(0x00, 0xAD))
            .with_pre_validation(|cpu| {
                cpu.regs["r1"] = 0xDE00;
            })
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(0), 0xAD);
                assert_eq!(cpu.mc.read(1), 0xDE);
            });

        // Test adding lit to *lit
        CpuTester::new(add(lit_star(0x00), lit(0xDE00)))
            .with_dram_seeder(|mc| mc.write(0x00, 0xAD))
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(0), 0xAD);
                assert_eq!(cpu.mc.read(1), 0xDE);
            });

        // Test adding reg to *lit
        CpuTester::new(add(lit_star(0x00), r("r0")))
            .with_dram_seeder(|mc| mc.write(0x00, 0xAD))
            .with_pre_validation(|cpu| {
                cpu.regs["r0"] = 0xDE00;
            })
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(0), 0xAD);
                assert_eq!(cpu.mc.read(1), 0xDE);
            });

        // Test adding *reg to *reg
        CpuTester::new(add(r_star("r0"), r_star("r1")))
            .with_dram_seeder(|mc| {
                mc.write(0x00, 0xAD);
                mc.write(0x03, 0xDE);
            })
            .with_pre_validation(|cpu| {
                cpu.regs["r0"] = 0x00;
                cpu.regs["r1"] = 0x02;
            })
            .with_post_validation(|cpu| {
                let entry = cpu.cache.lookup(0).expect("Cache entry not present");
                assert_eq!(entry, 0xDEAD);

                assert_eq!(cpu.mc.read(0), 0xAD);
                assert_eq!(cpu.mc.read(1), 0xDE);
            });
    }
}
