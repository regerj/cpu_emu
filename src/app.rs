use std::{
    env,
    fs::File,
    io::{
        BufReader,
        Read,
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
    console::{
        Console,
        ConsoleHandle,
    },
    cpu::Cpu,
    mem::{
        Dram,
        DramHandle,
        MemoryController,
        Sram,
    },
};

pub struct App {
    cpu: Cpu,
    dram_handle: DramHandle,
    console: ConsoleHandle,
}

impl App {
    pub fn new() -> Result<Self> {
        let args = env::args();
        let sram_path = PathBuf::from(args.into_iter().nth(1).expect("No path to binary provided"));
        let sram_file = File::open(&sram_path).expect("Cannot find binary file");
        let mut sram_bytes = Vec::new();
        BufReader::new(sram_file).read_to_end(&mut sram_bytes)?;

        // # Safety
        // Safe to unwrap here beacuse we have just asserted and resized.
        assert!(
            sram_bytes.len() <= 0x0FFF,
            "SRAM file is too long to fit in allocated address space"
        );
        sram_bytes.resize(0x0FFF, 0x6B);
        let sram = Sram::new(sram_bytes.try_into().unwrap());

        let (memory, dram_radio) = Dram::new();
        let (console, console_ep) = Console::new();
        let mut mem_ctrl = MemoryController::new();

        mem_ctrl.reg_mem_ep(dram_radio);
        mem_ctrl.reg_mem_ep(sram);
        mem_ctrl.reg_mem_ep(console_ep);

        let cpu = Cpu::new(mem_ctrl);

        // Start our HW blocks
        let h_mem = memory.dispatch();
        let h_console = console.dispatch();

        Ok(Self {
            cpu,
            dram_handle: h_mem,
            console: h_console,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let mut terminal = ratatui::init();

        loop {
            terminal.draw(|frame| self.render_ui(frame))?;
            self.cpu.cache.clear_highlights();
            self.cpu.regs.clear_highlights();
            self.handle_input()?;
            if self.cpu.execute().is_none() {
                break;
            }
        }

        ratatui::restore();

        Ok(())
    }

    fn render_ui(&self, frame: &mut Frame) {
        let chunks =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(3)]).split(frame.area());
        let (main_chunk, term_chunk) = (chunks[0], chunks[1]);
        let chunks = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_chunk);
        let (cpu_chunk, dram_chunk) = (chunks[0], chunks[1]);
        frame.render_widget(&self.cpu, cpu_chunk);
        frame.render_widget(&self.dram_handle, dram_chunk);
        frame.render_widget(&self.console, term_chunk);
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
