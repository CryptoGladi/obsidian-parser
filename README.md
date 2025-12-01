# obsidian-parser
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/obsidian-parser.svg)](https://crates.io/crates/obsidian-parser)
[![Docs.rs](https://docs.rs/obsidian-parser/badge.svg)](https://docs.rs/obsidian-parser)
[![Rust](https://img.shields.io/badge/Rust-orange.svg)](https://www.rust-lang.org)

Blazingly fast Rust library for parsing and analyzing [Obsidian](https://obsidian.md) vaults.
## Features
- ⚡ **High Performance**: Parses 1000+ notes in under 3ms
- 🧠 **Knowledge Graphs**: Built-in integration with [`petgraph`](https://docs.rs/petgraph/latest/petgraph) for advanced analysis
- 🧩 **Flexible API**: Supports both in-memory and on-disk note representations
- 🔍 **Frontmatter Parsing**: Extract YAML properties with [`serde`](https://docs.rs/serde/latest/serde) compatibility
- 🌐 **Link Analysis**: Identify connections between notes
- 👾 **WebAssembly Support**: Add `obsidian-parser` to your Obsidian plugins
## Quick Start
Add to `Cargo.toml`:
```toml
[dependencies]
obsidian-parser = "0.9"
```
### Basic Usage
* Basic Parsing
```rust
use obsidian_parser::prelude::*;
use serde::Deserialize;

// Parse single file with `HashMap`
let note_hashmap = NoteInMemory::from_file_default("note.md").unwrap();
println!("Content: {}", note_hashmap.content().unwrap());
println!("Properties: {:#?}", note_hashmap.properties().unwrap().unwrap());

// Parse single file with custom struct
#[derive(Clone, Deserialize)]
struct NoteProperties {
    created: String,
    tags: Vec<String>,
    priority: u8,
}
let note_with_serde: NoteInMemory<NoteProperties> = NoteInMemory::from_file("note.md").unwrap();
```
* Vault Analysis
```rust
use obsidian_parser::prelude::*;

// Load entire vault
let options = VaultOptions::new("/path/to/vault");
let vault: VaultInMemory = VaultBuilder::new(&options)
    .into_iter()
    .filter_map(Result::ok)
    .build_vault(&options)
    .unwrap();

// Check for duplicate note names
if !vault.have_duplicates_notes_by_name() {
    eprintln!("Duplicate note names detected!");
}

// Access parsed notes
for note in vault.notes() {
  println!("Note: {:?}", note);
}
```
* Graph Analysis (requires [`petgraph`](https://docs.rs/petgraph/latest/petgraph) feature)
```rust
#[cfg(feature = "petgraph")]
{
    use obsidian_parser::prelude::*;
    use petgraph::dot::{Dot, Config};
    let options = VaultOptions::new("/path/to/vault");
    let vault: VaultInMemory = VaultBuilder::new(&options)
        .into_iter()
        .filter_map(Result::ok)
        .build_vault(&options)
        .unwrap();
    let graph = vault.get_digraph().unwrap();
    
    // Export to Graphviz format
    println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
    
    // Find most connected note
    let most_connected = graph.node_indices()
        .max_by_key(|n| graph.edges(*n).count())
        .unwrap();
    println!("Knowledge hub: {:?}", graph[most_connected]);
}
```
## Example: Analyze Knowledge Connectivity
Included example `analyzer` calculates connected components in your Obsidian vault's knowledge graph:

```bash
cargo run --example analyzer --release --features="petgraph rayon" -- --path="Path to Obsidian vault"
```
## Performance
My PC AMD Ryzen 5 3600X with `NVMe` SSD
| Operation                | Time       |
|--------------------------|------------|
| Vault initialization     | 739.35 µs  |
| Graph construction       | 1.22 ms    |
| Peak memory usage        | 900 KiB    |
## License
MIT © [CryptoGladi](https://github.com/CryptoGladi)
