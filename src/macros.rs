#[macro_export]
macro_rules! cache_aligned {
    ($v:expr) => {
        $v & common::cfg::Word::MAX << (1 * std::mem::size_of::<common::cfg::CacheLine>() / 2)
    };
}

#[macro_export]
macro_rules! is_cache_aligned {
    ($v:expr) => {
        $v & !(common::cfg::Word::MAX << (1 * std::mem::size_of::<common::cfg::CacheLine>() / 2))
            == 0
    };
}

#[macro_export]
macro_rules! is_word_aligned {
    ($v:expr) => {
        $v & !(common::cfg::Word::MAX << (1 * std::mem::size_of::<common::cfg::Word>() / 2)) == 0
    };
}
