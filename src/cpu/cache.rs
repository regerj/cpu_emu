use std::fmt::Display;

use bitfield_struct::bitfield;
use common::cfg::{CHANGE_STYLE, CONFIG, CONST_CONFIG, CacheLine};
use rand::Rng;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::Widget,
};

use crate::{
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(cache);

#[derive(Debug)]
pub struct Cache {
    inner: [[Option<CacheEntry>; CONST_CONFIG.cache.ways]; CONST_CONFIG.cache.sets],
    highlighted: Vec<(usize, usize)>,
}

impl Cache {
    pub fn new() -> Self {
        telemetry_init!();
        Self {
            inner: [[None; CONST_CONFIG.cache.ways]; CONST_CONFIG.cache.sets],
            highlighted: vec![],
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

    /// Insert a cache line into the cache.
    ///
    /// The address does not necessarily need to be cache aligned, but the offset will be ignored.
    /// If you want to store a byte at a non-cache aligned address in the cache, you are responsible
    /// for shifting the byte to the correct position within the cache line prior to calling this
    /// function.
    pub fn insert(&mut self, addr: impl Into<CacheAddr>, value: CacheLine) {
        telemetry_log!(CONFIG.cycles.l1_cache_write);
        let addr: CacheAddr = addr.into();
        let new_entry = CacheEntry {
            tag: addr.tag(),
            val: value,
        };
        let set = &mut self.inner[addr.index()];

        // If we have a match
        if let Some((i, entry)) = set.iter_mut().enumerate().find(|(_, entry)| {
            let Some(entry) = entry else {
                return false;
            };

            entry.tag == addr.tag()
        }) {
            *entry = Some(new_entry);
            self.highlighted.push((addr.index(), i));
        } else if let Some((i, entry)) = set
            .iter_mut()
            .enumerate()
            .find(|(_, entry)| entry.is_none())
        {
            *entry = Some(new_entry);
            self.highlighted.push((addr.index(), i));
        } else {
            // Otherwise, evict randomly :D
            let mut rng = rand::rng();
            let which = (rng.next_u32() & 0b1) as usize;
            self.inner[addr.index()][which] = Some(new_entry);
            self.highlighted.push((addr.index(), which));
        }
    }

    pub fn clear_highlights(&mut self) {
        self.highlighted.clear();
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
                Span::from(
                    e.map(|e| format!("[{e}]"))
                        .unwrap_or_else(|| "[----------]".to_string()),
                )
            };

            let first = if self.highlighted.contains(&(idx, 0)) {
                fmt(set[0]).style(*CHANGE_STYLE)
            } else {
                fmt(set[0])
            };

            let second = if self.highlighted.contains(&(idx, 1)) {
                fmt(set[1]).style(*CHANGE_STYLE)
            } else {
                fmt(set[1])
            };

            let seperator = Span::from(" | ");
            Line::from(vec![
                Span::from(format!("{idx:>3}")),
                seperator.clone(),
                first,
                seperator,
                second,
            ])
        }));

        Paragraph::new(lines)
            .block(Block::bordered().title("L1 Cache"))
            .render(area, buf);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[cfg(test)]
mod tests {
    use crate::cpu::cache::{
        Cache,
        CacheAddr,
        CacheEntry,
    };

    #[test]
    fn test_lookup() {
        let mut cache = Cache::new();
        let addr0 = CacheAddr::from(0x0);
        let addr1 = CacheAddr::from(0x1);

        assert!(cache.lookup(addr0).is_none());
        assert!(cache.lookup(addr1).is_none());

        cache.inner[addr0.index()][0] = Some(CacheEntry {
            tag: addr0.tag(),
            val: 0xDEAD,
        });

        let val0 = cache.lookup(addr0).unwrap();
        assert_eq!(val0, 0xDEAD);

        let val1 = cache.lookup(addr1).unwrap();
        assert_eq!(val1, 0xDEAD);
    }

    #[test]
    fn test_insert() {
        let mut cache = Cache::new();
        let addr0 = CacheAddr::from(0x0);

        cache.insert(addr0, 0xDEAD);

        assert_eq!(
            cache.inner[addr0.offset()][0],
            Some(CacheEntry {
                tag: addr0.tag(),
                val: 0xDEAD
            })
        );
    }
}
