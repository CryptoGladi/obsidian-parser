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

#[must_use]
pub(crate) fn is_error<E>(error: impl std::error::Error) -> bool
where
    E: std::error::Error + 'static,
{
    let mut source = error.source();
    while let Some(err) = source {
        if err.downcast_ref::<E>().is_some() {
            return true;
        }

        source = err.source();
    }

    false
}
