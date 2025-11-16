//! It is module **only** for test!

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
