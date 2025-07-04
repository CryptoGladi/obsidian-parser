#[cfg(feature = "logging")]
pub(crate) fn init_test_logger() {
    use log::LevelFilter;
    use std::sync::Once;

    static INIT_LOGGER: Once = Once::new();

    INIT_LOGGER.call_once(|| {
        env_logger::Builder::new()
            .filter_level(LevelFilter::Debug)
            .parse_env("RUST_LOG")
            .is_test(true)
            .try_init()
            .expect("Failed to initialize logger");
    });
}

#[cfg(not(feature = "logging"))]
pub(crate) fn init_test_logger() {}
