use std::{
    fmt::{
        Debug,
        Display,
    },
    ops::{
        Add,
        AddAssign,
        Sub,
    },
};

use common::cfg::{
    CONFIG,
    Word,
};
use crossbeam::channel;
use log::debug;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{
        Line,
        Span,
        Text,
    },
    widgets::{
        Paragraph,
        Widget,
    },
};

use crate::{
    CHANGE_STYLE,
    block::Block,
    telemetry_init,
    telemetry_log,
    telemetry_module,
};

telemetry_module!(dram);

#[derive(Clone, Copy)]
pub struct MemoryRegion {
    begin: PhysAddr,
    length: Offset,
}

impl MemoryRegion {
    pub const fn new(begin: PhysAddr, len: Offset) -> Self {
        Self { begin, length: len }
    }

    pub fn contains(&self, addr: PhysAddr) -> bool {
        addr >= self.begin && addr < self.begin + self.length
    }
}

pub trait MemoryFabricEndpoint {
    fn id(&self) -> Option<String>;
    fn region(&self) -> MemoryRegion;
    fn read_byte(&self, addr: PhysAddr) -> u8;
    fn write_byte(&mut self, addr: PhysAddr, val: u8);
    fn kill(&self);
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Offset(Word);

impl Offset {
    pub const fn new(val: Word) -> Self {
        Self(val)
    }

    pub const fn into_raw(self) -> Word {
        self.0
    }
}

impl From<Word> for Offset {
    fn from(value: Word) -> Self {
        Self(value)
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct PhysAddr(Word);

impl PhysAddr {
    pub const fn new(val: Word) -> Self {
        Self(val)
    }

    pub const fn into_raw(self) -> Word {
        self.0
    }

    pub const fn is_word_aligned(&self) -> bool {
        self.0 & !(common::cfg::Word::MAX << (std::mem::size_of::<common::cfg::Word>() / 2)) == 0
    }
}

impl Display for PhysAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{:04X}", self.0)
    }
}

impl AddAssign<Offset> for PhysAddr {
    fn add_assign(&mut self, rhs: Offset) {
        self.0 += rhs.0;
    }
}

impl Add<Offset> for PhysAddr {
    type Output = PhysAddr;

    fn add(self, rhs: Offset) -> Self::Output {
        Self(self.0.wrapping_add(rhs.0))
    }
}

impl Sub<Offset> for PhysAddr {
    type Output = PhysAddr;

    fn sub(self, rhs: Offset) -> Self::Output {
        Self(self.0.wrapping_sub(rhs.0))
    }
}

impl Sub<PhysAddr> for PhysAddr {
    type Output = Offset;

    fn sub(self, rhs: PhysAddr) -> Self::Output {
        assert!(self.0 >= rhs.0);
        Offset(self.0.sub(rhs.0))
    }
}

impl From<Word> for PhysAddr {
    fn from(value: Word) -> Self {
        Self(value)
    }
}

impl From<PhysAddr> for Word {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

pub enum MemoryOps {
    Read(Offset),
    Write(Offset, u8),
    Kill,
}

pub struct MemoryController {
    eps: Vec<Box<dyn MemoryFabricEndpoint>>,
}

impl Debug for MemoryController {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Memory Controller EPs:")?;

        for ep in &self.eps {
            let region = ep.region();
            write!(
                f,
                "\t{}: [{}-{}]",
                ep.id().unwrap_or("UNKNOWN".to_string()),
                region.begin,
                region.begin + region.length
            )?;
        }

        Ok(())
    }
}

impl MemoryController {
    pub fn new() -> Self {
        Self { eps: Vec::new() }
    }

    /// Register a memory endpoint with the controller.
    ///
    /// The type system forces that you specify a region in the physical address space that your
    /// endpoint will occupy. This is not enforced to not overlap with already registered endpoints.
    /// In this case the first endpoint that contains the requested address will be used.
    pub fn reg_mem_ep(&mut self, ep: impl MemoryFabricEndpoint + 'static) {
        self.eps.push(Box::new(ep));
    }

    /// Read a byte from memory.
    ///
    /// Address does not need to be aligned.
    pub fn read(&self, address: PhysAddr) -> u8 {
        telemetry_log!(CONFIG.cycles.dram_read);
        self.sideband_read(address)
    }

    /// Write a byte to main memory.
    ///
    /// Address does not need to be aligned.
    pub fn write(&mut self, address: PhysAddr, value: u8) {
        telemetry_log!(CONFIG.cycles.dram_write);

        self.eps
            .iter_mut()
            .find(|ep| ep.region().contains(address))
            .expect("Attempt to access unmapped physical address")
            .write_byte(address, value);
    }

    /// Read a byte from memory without affecting telemetry or state.
    pub fn sideband_read(&self, addr: PhysAddr) -> u8 {
        self.eps
            .iter()
            .find(|ep| ep.region().contains(addr))
            .expect("Attempt to access unmapped physical address")
            .read_byte(addr)
    }

    pub fn kill(&self) {
        for ep in &self.eps {
            ep.kill();
        }
    }
}

#[derive(Debug)]
pub struct Sram {
    inner: [u8; 0x0FFF],
}

impl Sram {
    const REGION: MemoryRegion = MemoryRegion::new(PhysAddr(0xF000), Offset(0x0FFF));

    pub fn new(inner: [u8; 0x0FFF]) -> Self {
        Self { inner }
    }

    fn normalize_addr(addr: PhysAddr) -> Offset {
        addr - Self::REGION.begin
    }
}

impl MemoryFabricEndpoint for Sram {
    fn id(&self) -> Option<String> {
        Some("SRAM".to_string())
    }

    fn region(&self) -> MemoryRegion {
        Self::REGION
    }

    fn read_byte(&self, addr: PhysAddr) -> u8 {
        self.inner[Self::normalize_addr(addr).into_raw() as usize]
    }

    fn write_byte(&mut self, addr: PhysAddr, val: u8) {
        self.inner[Self::normalize_addr(addr).into_raw() as usize] = val;
    }

    fn kill(&self) {}
}

pub struct DramRadio {
    tx: channel::Sender<MemoryOps>,
    rx: channel::Receiver<Option<u8>>,
}

impl DramRadio {
    const REGION: MemoryRegion = MemoryRegion::new(PhysAddr(0x0000), Offset(0xF000));

    fn normalize_addr(addr: PhysAddr) -> Offset {
        addr - Self::REGION.begin
    }
}

impl MemoryFabricEndpoint for DramRadio {
    fn id(&self) -> Option<String> {
        Some("DRAM".to_string())
    }

    fn region(&self) -> MemoryRegion {
        MemoryRegion::new(PhysAddr(0x0000), Offset(0xF000))
    }

    fn read_byte(&self, addr: PhysAddr) -> u8 {
        self.tx
            .send(MemoryOps::Read(Self::normalize_addr(addr)))
            .expect("Panic in memory fabric");
        self.rx
            .recv()
            .expect("Panic in memory fabric")
            .expect("Empty response on memory fabric")
    }

    fn write_byte(&mut self, addr: PhysAddr, val: u8) {
        self.tx
            .send(MemoryOps::Write(Self::normalize_addr(addr), val))
            .expect("Panic in memory fabric");
        assert!(self.rx.recv().expect("Panic in memory fabric").is_none());
    }

    fn kill(&self) {
        self.tx
            .send(MemoryOps::Kill)
            .expect("Panic in memory fabric");
        assert!(self.rx.recv().expect("Panic in memory fabric").is_none());
    }
}

#[derive(Debug)]
pub struct Dram {
    inner: Vec<u8>,
    tx: channel::Sender<Option<u8>>,
    rx: channel::Receiver<MemoryOps>,
    mirror: Option<channel::Sender<MemoryOps>>,
}

impl Dram {
    pub fn new() -> (Self, DramRadio) {
        telemetry_init!();
        let (op_tx, op_rx) = channel::unbounded();
        let (resp_tx, resp_rx) = channel::unbounded();
        let radio = DramRadio {
            tx: op_tx,
            rx: resp_rx,
        };
        (
            Self {
                inner: vec![0; Word::MAX as usize + 1],
                tx: resp_tx,
                rx: op_rx,
                mirror: None,
            },
            radio,
        )
    }

    pub fn mirror(&mut self) -> DramMirror {
        let (mirror_tx, mirror_rx) = crossbeam::channel::unbounded();
        self.mirror = Some(mirror_tx);
        DramMirror {
            inner: self.inner.clone(),
            rx: mirror_rx,
            highlights: Vec::new(),
        }
    }

    fn read_byte(&self, addr: Offset) -> u8 {
        self.inner[addr.into_raw() as usize]
    }

    fn write_byte(&mut self, addr: Offset, value: u8) {
        self.inner[addr.into_raw() as usize] = value;
    }
}

impl Block for Dram {
    fn dispatch(mut self) {
        loop {
            let op = self.rx.recv().expect("Panic in memory fabric");
            match op {
                MemoryOps::Read(addr) => {
                    debug!("DRAM reading address {addr}");
                    self.tx
                        .send(Some(self.read_byte(addr)))
                        .expect("Panic in memory fabric");
                }
                MemoryOps::Write(addr, value) => {
                    debug!("DRAM writing address {addr} with value 0x{value:04x}");
                    self.write_byte(addr, value);
                    self.tx.send(None).expect("Panic in memory fabric");

                    // Replicate our write op to the mirror
                    if let Some(mirror) = self.mirror.as_ref() {
                        mirror.send(op).expect("Panic in memory mirror");
                    }
                }
                MemoryOps::Kill => {
                    self.tx.send(None).expect("Panic in memory fabric");
                    return;
                }
            }
        }
    }
}

pub struct DramMirror {
    inner: Vec<u8>,
    highlights: Vec<usize>,
    rx: channel::Receiver<MemoryOps>,
}

impl DramMirror {
    pub fn update(&mut self) {
        while let Ok(op) = self.rx.try_recv() {
            if let MemoryOps::Write(addr, value) = op {
                self.inner[addr.into_raw() as usize] = value;
                self.highlights.push(addr.into_raw() as usize);
            }
        }
    }

    pub fn clear_highlights(&mut self) {
        self.highlights.clear();
    }
}

impl Widget for &DramMirror {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const BYTES_PER_ROW: usize = 16;
        // TODO this some absolute slop, idk why its writing direct to buffer...
        // rewrite it as a text containing lines containing spans for each byte like a well adjusted
        // software engineer not some cretin
        let block = ratatui::widgets::Block::bordered().title("DRAM");
        let rows = self.inner.chunks(BYTES_PER_ROW);
        let mut text = Text::default();
        for (i, row) in rows.enumerate() {
            let offset = i * BYTES_PER_ROW;
            let mut line = Line::from(format!("{offset:04X}: "));

            for (j, byte) in row.iter().enumerate() {
                let span = if self.highlights.contains(&(offset + j)) {
                    Span::from(format!("{byte:02X}")).style(*CHANGE_STYLE)
                } else {
                    Span::from(format!("{byte:02X}"))
                };
                line.push_span(span);
                line.push_span(Span::from(" "));
            }

            text.push_line(line);
        }

        Paragraph::new(text).block(block).render(area, buf);
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        block::Block,
        mem::{
            Dram,
            MemoryFabricEndpoint,
            PhysAddr,
        },
    };

    #[test]
    fn test_read() {
        let (mut dram, mc) = Dram::new();
        dram.inner[0] = 0xDE;
        dram.inner[1] = 0xAD;
        dram.inner[2] = 0xBE;
        dram.inner[3] = 0xEF;

        let dram_handle = std::thread::spawn(move || dram.dispatch());

        assert_eq!(mc.read_byte(PhysAddr(0x0)), 0xDE);
        assert_eq!(mc.read_byte(PhysAddr(0x1)), 0xAD);
        assert_eq!(mc.read_byte(PhysAddr(0x2)), 0xBE);
        assert_eq!(mc.read_byte(PhysAddr(0x3)), 0xEF);

        mc.kill();
        dram_handle.join().unwrap();
    }

    #[test]
    fn test_write() {
        let (dram, mut mc) = Dram::new();
        let dram_handle = std::thread::spawn(move || dram.dispatch());

        mc.write_byte(PhysAddr(0x0), 0xDE);
        mc.write_byte(PhysAddr(0x1), 0xAD);
        mc.write_byte(PhysAddr(0x2), 0xBE);
        mc.write_byte(PhysAddr(0x3), 0xEF);

        assert_eq!(mc.read_byte(PhysAddr(0x0)), 0xDE);
        assert_eq!(mc.read_byte(PhysAddr(0x1)), 0xAD);
        assert_eq!(mc.read_byte(PhysAddr(0x2)), 0xBE);
        assert_eq!(mc.read_byte(PhysAddr(0x3)), 0xEF);

        mc.kill();
        dram_handle.join().unwrap();
    }
}
