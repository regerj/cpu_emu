use std::{
    collections::HashMap,
    ops::{
        Index,
        IndexMut,
    },
};

use bitfield_struct::bitfield;
use common::{
    cfg::Word,
    isa::Register,
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
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

use crate::CHANGE_STYLE;

#[bitfield(u16)]
pub struct StatusRegister {
    pub zero: bool,
    #[bits(15)]
    __: u16,
}

#[derive(Debug)]
pub struct Regs {
    gpr: HashMap<Register, Reg>,
    cpu: HashMap<Register, Reg>,
}

impl Regs {
    pub fn new() -> Self {
        Self {
            gpr: HashMap::from([
                (Register::R0, Reg::new()),
                (Register::R1, Reg::new()),
                (Register::R2, Reg::new()),
                (Register::R3, Reg::new()),
            ]),
            cpu: HashMap::from([
                (Register::IP, Reg::new().with(0xF000)),
                (Register::ST, Reg::new()),
                (Register::SB, Reg::new().with(0x0100)),
                (Register::SP, Reg::new().with(0x0100)),
                (Register::IT, Reg::new().with(0)),
            ]),
        }
    }

    pub fn clear_highlights(&mut self) {
        self.gpr
            .values_mut()
            .for_each(|reg| reg.highlighted = false);
        self.cpu
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
    const fn new() -> Self {
        Self {
            val: 0,
            highlighted: false,
        }
    }

    const fn with(mut self, val: Word) -> Self {
        self.val = val;
        self
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

impl Widget for &Regs {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut lines = vec![
            Line::from("General Purpose Registers"),
            Line::from(vec![Span::styled(
                "Reg │ Value",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("────┼────────"),
        ];

        for (name, reg) in self.gpr.iter() {
            let name = Span::from(name.to_string());
            let spacer = Span::from("  │ ");
            let reg: Span = reg.into();
            lines.push(Line::from(vec![name, spacer, reg]));
        }

        lines.append(&mut vec![
            Line::from("CPU Registers"),
            Line::from(vec![Span::styled(
                "Reg │ Value",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("────┼────────"),
        ]);

        for (name, reg) in self.cpu.iter() {
            let name = Span::from(name.to_string());
            let spacer = Span::from("  │ ");
            let reg: Span = reg.into();
            lines.push(Line::from(vec![name, spacer, reg]));
        }

        Paragraph::new(lines)
            .block(Block::bordered().title("Registers"))
            .render(area, buf);
    }
}

impl Index<Register> for Regs {
    type Output = Word;
    fn index(&self, index: Register) -> &Self::Output {
        if let Some(val) = self.gpr.get(&index) {
            &val.val
        } else if let Some(val) = self.cpu.get(&index) {
            &val.val
        } else {
            panic!("No register by that name")
        }
    }
}

impl IndexMut<Register> for Regs {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        if let Some(val) = self.gpr.get_mut(&index) {
            val.highlighted = true;
            &mut val.val
        } else if let Some(val) = self.cpu.get_mut(&index) {
            val.highlighted = true;
            &mut val.val
        } else {
            panic!("No register by that name")
        }
    }
}
