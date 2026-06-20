#[macro_export]
macro_rules! aligned {
    ($v:expr) => {
        $v & !($crate::WORD::MAX << (1 * std::mem::size_of::<$crate::cache::CacheLine>() / 2))
    };
}
