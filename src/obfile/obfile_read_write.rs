use super::{ObFileRead, ObFileWrite};
use crate::obfile::parser;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub trait ObFileReadWrite: ObFileRead + ObFileWrite
where
    Self::Properties: Serialize + DeserializeOwned,
    Self::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
{
}

impl<T> ObFileReadWrite for T
where
    T: ObFileRead + ObFileWrite,
    T::Properties: Serialize + DeserializeOwned,
    T::Error: From<std::io::Error> + From<serde_yml::Error> + From<parser::Error>,
{
}
