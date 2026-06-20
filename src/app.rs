use std::{
    env,
    fs::File,
    io::{
        BufRead,
        BufReader,
    },
    path::PathBuf,
};

use anyhow::{
    Result,
    bail,
};
use ratatui::{
    Frame,
    crossterm::event::{
        self,
        Event,
        KeyCode,
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
    mem::{
        Dram,
        DramMirror,
    },
    ops::Operation,
};

pub struct App {
    cpu: Cpu,
    dram_mirror: DramMirror,
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
            .map(|instruction| Operation::try_from(instruction?.as_str()))
            .collect::<Result<Vec<Operation>, _>>()?;

        let (mut memory, mc) = Dram::new();
        let cpu = Cpu::new(mc, instructions.into_iter());
        let dram_mirror = memory.mirror();

        // Start our DRAM block
        std::thread::spawn(|| memory.dispatch());
        Ok(Self { cpu, dram_mirror })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        loop {
            terminal.draw(|frame| self.render_ui(frame))?;
            self.cpu.cache.clear_highlights();
            self.cpu.regs.clear_highlights();
            self.dram_mirror.clear_highlights();
            self.handle_input()?;
            if self.cpu.execute().is_none() {
                break;
            }

            // DRAM mirror updates from writes
            self.dram_mirror.update();
        }

        ratatui::restore();

        Ok(())
    }

    fn render_ui(&self, frame: &mut Frame) {
        let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.area());
        let (cpu_chunk, dram_chunk) = (chunks[0], chunks[1]);
        frame.render_widget(&self.cpu, cpu_chunk);
        frame.render_widget(&self.dram_mirror, dram_chunk);
    }

    fn handle_input(&mut self) -> Result<()> {
        match event::read()? {
            // If we have a pressed key and it is unhandled (bad input), try again recursively
            Event::Key(key_event)
                if key_event.kind.is_press() && self.handle_key_event(key_event).is_err() =>
            {
                self.handle_input()?;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Char('n') => {}
            _ => bail!("Invalid keystroke"),
        }
        Ok(())
    }
}
