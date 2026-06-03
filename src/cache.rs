use bitfield_struct::bitfield;

use crate::{
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(cache);

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

    pub fn lookup(&self, addr: &CacheAddr) -> Option<u16> {
        telemetry_log!(4);
        self.inner[addr.index()].iter().find_map(|entry| {
            let entry = (*entry)?;
            if entry.tag == addr.tag() {
                Some(entry.val)
            } else {
                None
            }
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CacheEntry {
    tag: u8,
    val: u16,
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
