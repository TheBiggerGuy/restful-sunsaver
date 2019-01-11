macro_rules! conv_100_2_15_scale {
    ($expression:expr) => {
        (f32::from($expression) * 100.0) / 32768.0
    };
}

macro_rules! conv_7916_2_15_scale {
    ($expression:expr) => {
        (f32::from($expression) * 79.16) / 32768.0
    };
}
