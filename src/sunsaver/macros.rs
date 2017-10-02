macro_rules! conv_100_2_15_scale {
    ($expression:expr) => (
        (($expression as f32) * 100.0) / 32768.0
    )
}

macro_rules! conv_7916_2_15_scale {
    ($expression:expr) => (
        (($expression as f32) * 79.16) / 32768.0
    )
}
