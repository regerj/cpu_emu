#[macro_export]
macro_rules! cache_aligned {
    ($v:expr) => {
        $v & !($crate::cfg::Word::MAX << (1 * std::mem::size_of::<$crate::cfg::CacheLine>() / 2))
    };
}
