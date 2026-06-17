use std::{env, fs::File, io::{BufRead, BufReader}, path::PathBuf, time::Duration};

use anyhow::Result;

use crate::{block::Block, cpu::Cpu, mem::Dram, ops::Operation, ui::render};

pub struct App {
}

impl App {
    pub fn new() -> Self {
        Self {  }
    }
    
    pub fn run(&mut self) -> Result<()> {
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
        }

        let mut x = ratatui::init();
        x.draw(|frame| render(frame, &cpu.cache)).unwrap();
        std::thread::sleep(Duration::from_secs(12));
        ratatui::restore();

        Ok(())
    }
}
