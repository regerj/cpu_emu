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
use ratatui::{
    Frame,
    crossterm::event::{
        self,
        Event,
        KeyEvent,
    },
    layout::{
        Constraint,
        Layout,
    },
};

use crate::{
    block::Block,
    cpu::Cpu,
    mem::Dram,
    ops::Operation,
    ui::render,
};

pub struct App {
    cpu: Cpu,
}

impl App {
    pub fn new() -> Result<Self> {
        let args = env::args();
        let instructions =
            PathBuf::from(args.into_iter().nth(1).expect("No path to binary provided"));
        let binary = File::open(&instructions).expect("Cannot find binary file");
        let instructions = BufReader::new(binary);

        let instructions: Vec<Operation> = instructions
            .lines()
            .into_iter()
            .map(|instruction| Operation::try_from(instruction?.as_str()))
            .collect::<Result<Vec<Operation>, _>>()?;

        let (memory, mc) = Dram::new();
        let cpu = Cpu::new(mc, instructions.into_iter());

        // Start our DRAM block
        std::thread::spawn(|| memory.dispatch());
        Ok(Self { cpu })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        loop {
            terminal.draw(|frame| self.render_ui(frame))?;
            self.handle_input()?;
            if self.cpu.execute().is_none() {
                break;
            }
        }

        ratatui::restore();

        Ok(())
    }

    fn render_ui(&self, frame: &mut Frame) {
        let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
        frame.render_widget(&self.cpu.regs, chunks[0]);
        frame.render_widget(&self.cpu.cache, chunks[1]);
    }

    fn handle_input(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind.is_press() => {
                self.handle_key_event(key_event)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            _ => {}
        }
        Ok(())
    }
}
