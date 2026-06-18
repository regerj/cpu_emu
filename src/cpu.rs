telemetry_module!(cpu);

use std::{
    ops::{
        Index,
        IndexMut,
    },
    vec::IntoIter,
};

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
        let chunks = Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
        let (upper_chunk, lower_chunk) = (chunks[0], chunks[1]);

        let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(upper_chunk);
        let (left_chunk, right_chunk) = (chunks[0], chunks[1]);

        let mut iter = self.instructions.as_ref().iter();
        let mut instructions = vec![];
        if let Some(instruction) = iter.next() {
            instructions.push(Line::from(instruction.to_string()).style(Style::default().bold()));
        }

        iter.for_each(|elem| instructions.push(Line::from(elem.to_string()).style(Style::default().dim())));

        Paragraph::new(instructions).block(Block::bordered().title("Instructions")).render(left_chunk, buf);
        self.regs.render(right_chunk, buf);
        self.cache.render(lower_chunk, buf);
    }
}

#[derive(Debug, Default)]
pub struct Regs {
    r0: WORD,
    r1: WORD,
    r2: WORD,
    r3: WORD,
}

use ratatui::{
    buffer::Buffer,
    layout::{
        Constraint, Layout, Rect
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
        let lines = vec![
            Line::from(vec![Span::styled(
                "Reg │ Value",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("────┼────────"),
            Line::from(format!("R0  │ 0x{:02X}", self.r0)),
            Line::from(format!("R1  │ 0x{:02X}", self.r1)),
            Line::from(format!("R2  │ 0x{:02X}", self.r2)),
            Line::from(format!("R3  │ 0x{:02X}", self.r3)),
        ];

        Paragraph::new(lines)
            .block(Block::bordered().title("Registers"))
            .render(area, buf);
    }
}

impl Index<usize> for Regs {
    type Output = WORD;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.r0,
            1 => &self.r1,
            2 => &self.r2,
            3 => &self.r3,
            _ => panic!("Invalid register identifier"),
        }
    }
}

impl IndexMut<usize> for Regs {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.r0,
            1 => &mut self.r1,
            2 => &mut self.r2,
            3 => &mut self.r3,
            _ => panic!("Invalid register identifier"),
        }
    }
}

impl Cpu {
    pub fn new(mc: MemoryController, instructions: IntoIter<crate::ops::Operation>) -> Self {
        telemetry_init!();
        Self {
            regs: Regs::default(),
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
                        self.write_addr(dest.word, dest_val - src_word);
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
                        self.write_addr(dest.word, src_word);
                    }
                }
            }
        }

        Some(())
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
        self.mc.write(addr, value);
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
