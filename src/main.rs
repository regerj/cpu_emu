use anyhow::Result;

use crate::{
    app::App,
    telemetry::TELEMETRY,
};

mod app;
mod block;
mod cache;
mod cpu;
mod macros;
mod mem;
mod ops;
mod telemetry;
mod ui;

pub type WORD = u8;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run().unwrap();

    println!("{}", TELEMETRY.lock().expect("Telemetry poisoned"));

    Ok(())
}
