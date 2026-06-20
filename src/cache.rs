use std::fmt::Display;

use bitfield_struct::bitfield;
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::{
    cfg::{
        CONFIG,
        CONST_CONFIG,
        CacheLine,
    },
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(cache);

#[derive(Debug)]
pub struct Cache {
    inner: [[Option<CacheEntry>; CONST_CONFIG.cache.ways]; CONST_CONFIG.cache.sets],
}

impl Cache {
    pub fn new() -> Self {
        telemetry_init!();
        Self {
            inner: [[None; CONST_CONFIG.cache.ways]; CONST_CONFIG.cache.sets],
        }
    }

    pub fn lookup(&self, addr: impl Into<CacheAddr>) -> Option<CacheLine> {
        telemetry_log!(CONFIG.cycles.l1_cache_read);
        let addr: CacheAddr = addr.into();
        self.inner[addr.index()].iter().find_map(|entry| {
            let entry = (*entry)?;
            if entry.tag == addr.tag() {
                Some(entry.val)
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, addr: impl Into<CacheAddr>, value: CacheLine) {
        telemetry_log!(CONFIG.cycles.l1_cache_write);
        let addr: CacheAddr = addr.into();
        let new_entry = CacheEntry {
            tag: addr.tag(),
            val: value,
        };
        let set = &mut self.inner[addr.index()];

        // If we have a match
        if let Some(entry) = set.iter_mut().find(|entry| {
            let Some(entry) = entry else {
                return false;
            };

            entry.tag == addr.tag()
        }) {
            *entry = Some(new_entry);
        } else if let Some(entry) = set.iter_mut().find(|entry| entry.is_none()) {
            *entry = Some(new_entry);
        } else {
            // Otherwise, evict randomly :D
            let mut rng = rand::rng();
            let which = (rng.next_u32() & 0b1) as usize;
            self.inner[addr.index()][which] = Some(new_entry);
        }
    }
}

impl Widget for &Cache {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use ratatui::{
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
            },
        };

        let mut lines = vec![
            Line::from(vec![Span::styled(
                "Idx │ Way 0        │ Way 1",
                Style::default().add_modifier(Modifier::BOLD),
            )]),
            Line::from("────┼──────────────┼──────────────"),
        ];

        lines.extend(self.inner.iter().enumerate().map(|(idx, set)| {
            let fmt = |e: Option<CacheEntry>| {
                e.map(|e| format!("[{e}]"))
                    .unwrap_or_else(|| "[----]".to_string())
            };

            Line::from(format!(
                "{idx:>3} │ {:<12} │ {:<12}",
                fmt(set[0]),
                fmt(set[1]),
            ))
        }));

        Paragraph::new(lines)
            .block(Block::bordered().title("L1 Cache"))
            .render(area, buf);
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheEntry {
    tag: u16,
    val: CacheLine,
}

impl Display for CacheEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:x}:0x{:04x}", self.tag, self.val)
    }
}

#[bitfield(u16)]
pub struct CacheAddr {
    #[bits(1)]
    pub offset: usize,
    #[bits(3)]
    pub index: usize,
    #[bits(12)]
    pub tag: u16,
}
