//! `obsidian-parser` - Blazingly fast Rust library for parsing and analyzing Obsidian vaults
//!
//! Provides idiomatic APIs for:
//! - Parsing individual Obsidian notes with frontmatter properties
//! - Analyzing entire vaults as knowledge graphs
//! - Extracting semantic relationships between notes
//!
//! ## Key Features
//! * 🛡️ **100% Safe Rust** - Strictly forbids unsafe code (`#![forbid(unsafe_code)]`)
//! * ⚡ **High Performance** - Parses 1000 notes in <3ms
//! * 🕸️ **Knowledge Graphs** - Built-in petgraph integration for graph analysis (requires `petgraph` feature)
//!
//! ## Usage
//! Add to `Cargo.toml`:
//! ```toml
//! [dependencies]
//! obsidian-parser = { version = "0.1", features = ["petgraph", "rayon"] }
//! ```
//!
//! ## Examples
//!
//! ### Basic Parsing
//! ```no_run
//! use obsidian_parser::prelude::*;
//! use serde::Deserialize;
//!
//! // Parse single file with `HashMap`
//! let note_hashmap = ObFileInMemory::from_file_default("note.md").unwrap();
//!
//! println!("Content: {}", note_hashmap.content());
//! println!("Properties: {:#?}", note_hashmap.properties());
//!
//! // Parse single file with custom struct
//! #[derive(Clone, Default, Deserialize)]
//! struct NoteProperties {
//!     created: String,
//!     tags: Vec<String>,
//!     priority: u8,
//! }
//!
//! let note_with_serde: ObFileInMemory<NoteProperties> = ObFileInMemory::from_file("note.md").unwrap();
//! ```
//!
//! ### Vault Analysis
//! ```no_run
//! use obsidian_parser::prelude::*;
//!
//! // Load entire vault
//! let vault = Vault::open_default("/path/to/vault").unwrap();
//!
//! // Check for duplicate note names
//! if !vault.has_unique_filenames() {
//!     eprintln!("Duplicate note names detected!");
//! }
//! ```
//!
//! ### Graph Analysis (requires `petgraph` feature)
//! ```no_run
//! #[cfg(feature = "petgraph")]
//! {
//!     use obsidian_parser::prelude::*;
//!     use petgraph::dot::{Dot, Config};
//!
//!     let vault = Vault::open_default("/path/to/vault").unwrap();
//!     let graph = vault.get_digraph();
//!     
//!     // Export to Graphviz format
//!     println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
//!     
//!     // Find most connected note
//!     let most_connected = graph.node_indices()
//!         .max_by_key(|n| graph.edges(*n).count())
//!         .unwrap();
//!     println!("Knowledge hub: {}", graph[most_connected]);
//! }
//! ```
//!
//! ## Performance
//! Optimized for large vaults:
//! - 🚀 1000 files parsed in 2.7ms (avg)
//! - 💾 Peak memory: 900KB per 1000 notes
//!
//! Parallel processing via Rayon (enable `rayon` feature)
//!
//! ## Graph Features
//! When `petgraph` feature is enabled:
//! - Build directed/undirected knowledge graphs
//! - Analyze note connectivity
//! - Detect knowledge clusters
//! - Calculate centrality metrics
//!
//! Graph nodes use note names, edges represent links (`[[...]]`).

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::panic)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::unreadable_literal)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::as_conversions)]

pub mod error;
pub mod obfile;
pub mod vault;

#[cfg(test)]
pub(crate) mod test_utils;

pub mod prelude {
    pub use crate::obfile::obfile_in_memory::ObFileInMemory;
    pub use crate::obfile::obfile_on_disk::ObFileOnDisk;
    pub use crate::obfile::{ObFile, ObFileDefault};
    pub use crate::vault::Vault;
}
