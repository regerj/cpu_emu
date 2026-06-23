use std::{
    collections::HashMap,
    ops::{
        Index,
        IndexMut,
    },
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

use crate::cfg::{
    CHANGE_STYLE,
    Word,
};

#[derive(Debug)]
pub struct Regs {
    inner: HashMap<String, Reg>,
}

impl Regs {
    pub fn new() -> Self {
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
            let name = Span::from(name);
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
