use anyhow::Result;
use flexi_logger::{
    FileSpec,
    Logger,
    WriteMode,
};

use crate::{
    app::App,
    telemetry::TELEMETRY,
};

mod app;
mod block;
mod cpu;
mod macros;
mod mem;
mod telemetry;

fn init_logging() -> Result<()> {
    let _logger = Logger::try_with_str("debug")?
        .log_to_file(
            FileSpec::default()
                .directory("logs")
                .basename("emu")
                .suppress_timestamp(),
        )
        .write_mode(WriteMode::BufferAndFlush)
        .start()?;
    Ok(())
}

fn main() -> Result<()> {
    init_logging()?;

    let mut app = App::new()?;
    app.run().unwrap();

    println!("{}", TELEMETRY.lock().expect("Telemetry poisoned"));

    Ok(())
}
