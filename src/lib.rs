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
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod obfile;
pub mod prelude;
pub mod vault;

#[cfg(test)]
pub(crate) mod test_utils;
