use std::{
    collections::HashMap,
    ops::{
        Index,
        IndexMut,
    },
};

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

#[derive(Debug)]
pub struct Regs {
    inner: HashMap<Register, Reg>,
}

impl Regs {
    pub fn new() -> Self {
        Self {
            inner: HashMap::from([
                (Register::R0, Reg::new()),
                (Register::R1, Reg::new()),
                (Register::R2, Reg::new()),
                (Register::R3, Reg::new()),
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
        &self
            .inner
            .get(&index)
            .expect("No register by that name")
            .val
    }
}

impl IndexMut<Register> for Regs {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        let reg = self
            .inner
            .get_mut(&index)
            .expect("No register by that name");
        reg.highlighted = true;
        &mut reg.val
    }
}
