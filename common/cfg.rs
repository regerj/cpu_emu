use std::{
    fs::File,
    io::Read,
    path::Path,
    sync::LazyLock,
};

use ratatui::style::{
    Color,
    Style,
};
use serde::{
    Deserialize,
    Serialize,
};

pub type Word = u16;
pub type CacheLine = u16;

pub const CONST_CONFIG: CConfig = CConfig {
    cache: CacheCConfig { ways: 2, sets: 8 },
};

pub static CHANGE_STYLE: LazyLock<Style> =
    LazyLock::new(|| Style::default().bg(Color::Cyan).fg(Color::DarkGray));

pub struct CConfig {
    pub cache: CacheCConfig,
}

#[derive(Debug)]
pub struct CacheCConfig {
    pub ways: usize,
    pub sets: usize,
}

pub static CONFIG: LazyLock<Config> = LazyLock::new(|| {
    let config_path = Path::new("machine.toml");
    let mut config_str = String::new();

    File::open(config_path)
        .expect("Cannot open file: machine.toml")
        .read_to_string(&mut config_str)
        .expect("Cannot read from file: machine.toml");

    toml::from_str(&config_str).expect("Failed to deserialize machine configuration")
});

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub cycles: CyclesConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CyclesConfig {
    pub l1_cache_read: usize,
    pub l1_cache_write: usize,
    pub dram_read: usize,
    pub dram_write: usize,
}
