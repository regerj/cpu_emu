use std::{
    collections::HashMap,
    sync::{
        LazyLock,
        Mutex,
    },
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
