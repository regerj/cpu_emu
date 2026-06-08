use std::{
    collections::HashMap,
    fmt::Display,
    sync::{
        LazyLock,
        Mutex,
    },
};

use comfy_table::{
    Attribute,
    Color,
    ContentArrangement,
    Table,
    presets::UTF8_FULL,
};

pub struct TelemetryTracker {
    inner: HashMap<String, usize>,
}

impl TelemetryTracker {
    pub fn register_module(&mut self, name: &str) {
        self.inner.insert(name.to_string(), 0);
    }

    pub fn log_cycles(&mut self, name: &str, cycles: usize) {
        *self
            .inner
            .get_mut(name)
            .expect("Cannot log to non-existant module") += cycles;
    }
}

impl Display for TelemetryTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use comfy_table::Cell;
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(80)
            .set_header(vec![
                Cell::new("Module").add_attribute(Attribute::Bold),
                Cell::new("Cycles").add_attribute(Attribute::Bold),
            ]);

        let mut total = 0;
        for (module, tally) in self.inner.iter() {
            table.add_row(vec![Cell::new(module), Cell::new(tally)]);
            total += tally;
        }

        table.add_row(vec![
            Cell::new("Total").fg(Color::Blue),
            Cell::new(total).fg(Color::Blue),
        ]);

        write!(f, "{table}")
    }
}

pub static TELEMETRY: LazyLock<Mutex<TelemetryTracker>> = LazyLock::new(|| {
    Mutex::new(TelemetryTracker {
        inner: HashMap::new(),
    })
});

#[macro_export]
macro_rules! telemetry_module {
    ($i:ident) => {
        static THIS_MODULE: &str = stringify!($i);
    };
}

#[macro_export]
macro_rules! telemetry_init {
    () => {
        $crate::telemetry::TELEMETRY
            .lock()
            .expect("Panic in telemetry")
            .register_module(THIS_MODULE);
    };
}

#[macro_export]
macro_rules! telemetry_log {
    ($n:expr) => {
        $crate::telemetry::TELEMETRY
            .lock()
            .expect("Panic in telemetry")
            .log_cycles(THIS_MODULE, $n);
    };
}
