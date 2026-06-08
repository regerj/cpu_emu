use bitfield_struct::bitfield;
use rand::Rng;

use crate::{
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(cache);

pub type CACHE_LINE = u16;

/// The cache is a 1 level 2-way set associative cache.
///
/// It utilizes 2 byte cache line size and has a total capacity of 16 cache lines.
#[derive(Debug)]
pub struct Cache {
    inner: [[Option<CacheEntry>; 2]; 8],
}

impl Cache {
    pub fn new() -> Self {
        telemetry_init!();
        Self {
            inner: [[None; 2]; 8],
        }
    }

    pub fn lookup(&self, addr: impl Into<CacheAddr>) -> Option<CACHE_LINE> {
        telemetry_log!(4);
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

    pub fn insert(&mut self, addr: impl Into<CacheAddr>, value: CACHE_LINE) {
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

#[derive(Debug, Clone, Copy)]
pub struct CacheEntry {
    tag: u8,
    val: CACHE_LINE,
}

#[bitfield(u8)]
pub struct CacheAddr {
    #[bits(1)]
    pub offset: usize,
    #[bits(3)]
    pub index: usize,
    #[bits(4)]
    pub tag: u8,
}
