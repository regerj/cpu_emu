use std::{
    env,
    fs::File,
    io::{
        BufRead,
        BufReader,
    },
    path::PathBuf,
};

use anyhow::Result;

use crate::{
    block::Block,
    cpu::Cpu,
    mem::Dram,
    ops::Operation,
};

mod block;
mod cache;
mod cpu;
mod mem;
mod ops;
mod telemetry;

pub type WORD = u8;

fn main() -> Result<()> {
    let args = env::args();
    let binary = PathBuf::from(args.into_iter().nth(1).expect("No path to binary provided"));
    let binary = File::open(binary)?;
    let bufreader = BufReader::new(binary);

    let (memory, mc) = Dram::new();
    let mut cpu = Cpu::new(mc);

    // Start our DRAM block
    std::thread::spawn(|| memory.dispatch());

    for line in bufreader.lines() {
        let line = line?;
        cpu.execute(Operation::try_from(line.as_str())?);
        println!("{cpu:#?}");
    }

    Ok(())
}
