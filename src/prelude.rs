//! All prelude

pub use crate::note::note_in_memory::NoteInMemory;
pub use crate::note::note_on_disk::NoteOnDisk;
pub use crate::note::{Note, NoteDefault, NoteRead, NoteWrite};
pub use crate::vault::vault_open::{IteratorVaultBuilder, VaultBuilder, VaultOptions};
pub use crate::vault::{Vault, VaultInMemory, VaultOnDisk, VaultOnceCell};

#[cfg(feature = "rayon")]
pub use crate::vault::vault_open::ParallelIteratorVaultBuilder;
