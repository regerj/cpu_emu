#[macro_export]
macro_rules! cache_aligned {
    ($v:expr) => {
        $v & $crate::cfg::Word::MAX << (1 * std::mem::size_of::<$crate::cfg::CacheLine>() / 2)
    };
}

#[macro_export]
macro_rules! is_cache_aligned {
    ($v:expr) => {
        $v & !($crate::cfg::Word::MAX << (1 * std::mem::size_of::<$crate::cfg::CacheLine>() / 2))
            == 0
    };
}

#[macro_export]
macro_rules! is_word_aligned {
    ($v:expr) => {
        $v & !($crate::cfg::Word::MAX << (1 * std::mem::size_of::<$crate::cfg::Word>() / 2)) == 0
    };
}
