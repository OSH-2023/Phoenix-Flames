#[macro_export]
macro_rules! MASK {
    ($x:expr) => {
        (1u64 << ($x)) - 1u64
    };
}

#[macro_export]
macro_rules! BIT {
    ($x:expr) => {
        (1u64 << ($x))
    };
}