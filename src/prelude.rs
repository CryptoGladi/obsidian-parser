//! All prelude

pub use crate::note::note_in_memory::NoteInMemory;
//pub use crate::note::note_on_disk::NoteOnDisk;
//pub use crate::note::note_once_cell::NoteOnceCell;
//pub use crate::note::note_once_lock::NoteOnceLock;
pub use crate::note::{Note, NoteDefault, NoteFromFile, NoteFromReader, NoteFromString, NoteWrite};
pub use crate::vault::vault_open::{IteratorVaultBuilder, VaultBuilder, VaultOptions};
pub use crate::vault::{Vault, VaultInMemory, VaultOnDisk, VaultOnceCell, VaultOnceLock};

#[cfg(feature = "rayon")]
pub use crate::vault::vault_open::ParallelIteratorVaultBuilder;
