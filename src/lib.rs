//! `obsidian-parser` - Blazingly fast Rust library for parsing and analyzing [Obsidian](https://obsidian.md) vaults
//!
//! Provides idiomatic APIs for:
//! - Parsing individual Obsidian notes with frontmatter properties
//! - Analyzing entire vaults as knowledge graphs
//! - Extracting semantic relationships between notes
//!
//! ## Key Features
//! * ⚡ **High Performance**: Parses 1000+ notes in under 3ms
//! * 🧠 **Knowledge Graphs**: Built-in integration with [`petgraph`](https://docs.rs/petgraph/latest/petgraph) for advanced analysis
//! * 🧩 **Flexible API**: Supports both in-memory and on-disk note representations
//! * 🔍 **Frontmatter Parsing**: Extract YAML properties with [`serde`](https://docs.rs/serde/latest/serde) compatibility
//! * 🌐 **Link Analysis**: Identify connections between notes
//!
//! ## Usage
//! Add to `Cargo.toml`:
//! ```toml
//! [dependencies]
//! obsidian-parser = { version = "0.6", features = ["petgraph", "rayon"] }
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
//! println!("Content: {}", note_hashmap.content().unwrap());
//! println!("Properties: {:#?}", note_hashmap.properties().unwrap().unwrap());
//!
//! // Parse single file with custom struct
//! #[derive(Clone, Deserialize)]
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
//! if !vault.check_unique_note_name() {
//!     eprintln!("Duplicate note names detected!");
//! }
//!
//! // Access parsed files
//! for file in vault.files {
//!   println!("Note: {:?}", file.path());
//! }
//! ```
//!
//! ### Graph Analysis (requires [`petgraph`](https://docs.rs/petgraph/latest/petgraph) feature)
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
//! - 🚀 1000 files parsed in 2.6 ms (avg)
//! - 💾 Peak memory: 900KB per 1000 notes
//!
//! Parallel processing via Rayon (enable `rayon` feature)

//#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::cargo)]
#![warn(clippy::nursery)]
#![warn(clippy::perf)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::panic)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::unreadable_literal)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::as_conversions)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod obfile;
pub mod prelude;
pub mod vault;

#[cfg(test)]
pub(crate) mod test_utils;
