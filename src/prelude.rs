//! All prelude

pub use crate::obfile::obfile_in_memory::ObFileInMemory;
pub use crate::obfile::obfile_on_disk::ObFileOnDisk;
pub use crate::obfile::{ObFile, ObFileDefault, ObFileRead, ObFileWrite};
pub use crate::vault::vault_open::{FilesBuilder, IteratorFilesBuilder, VaultOptions};
pub use crate::vault::{Vault, VaultInMemory, VaultOnDisk};

#[cfg(feature = "rayon")]
pub use crate::vault::vault_open::ParallelIteratorFilesBuilder;
