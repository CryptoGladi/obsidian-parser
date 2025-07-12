# obsidian-parser
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/obsidian-parser.svg)](https://crates.io/crates/obsidian-parser)
[![Docs.rs](https://docs.rs/obsidian-parser/badge.svg)](https://docs.rs/obsidian-parser)
[![Rust](https://img.shields.io/badge/Rust-orange.svg)](https://www.rust-lang.org)

Blazingly fast Rust library for parsing and analyzing [Obsidian](https://obsidian.md) vaults.
## Features
- ⚡ **High Performance**: Parses 1000+ notes in under 3ms
- 🧠 **Knowledge Graphs**: Built-in integration with `petgraph` for advanced analysis
- 🧩 **Flexible API**: Supports both in-memory and on-disk note representations
- 🔍 **Frontmatter Parsing**: Extract YAML properties with Serde compatibility
- 🌐 **Link Analysis**: Identify connections between notes
## Quick Start
Add to `Cargo.toml`:
```toml
[dependencies]
obsidian-parser = "0.1"
```
### Basic Usage
*  Parsing
```rust
use obsidian_parser::prelude::*;
use serde::Deserialize;

// Parse single file with `HashMap`
let note_hashmap = ObFileInMemory::from_file_default("note.md").unwrap();

println!("Content: {}", note_hashmap.content());
println!("Properties: {:#?}", note_hashmap.properties());

// Parse single file with custom struct
#[derive(Clone, Default, Deserialize)]
struct NoteProperties {
    created: String,
     tags: Vec<String>,
     priority: u8,
}

let note_with_serde: ObFileInMemory<NoteProperties> = ObFileInMemory::from_file("note.md").unwrap();
```
* Vault
```rust
use obsidian_parser::prelude::*;
// Open vault (defaults to on-disk representation for efficiency)
let vault = Vault::open_default("/path/to/vault")?;
// Check for duplicate note names (critical for graph operations)
if !vault.has_unique_filenames() {
    eprintln!("Duplicate note names detected!");
}
// Access parsed files
for file in &vault.files {
    println!("Note: {:?}", file.path());
}
```
### Graph Analysis (requires `petgraph` feature)
Enable in `Cargo.toml`:
```toml
obsidian-parser = { version = "0.1", features = ["petgraph"] }
# obsidian-parser = { version = "0.1", features = ["petgraph", "rayon"] } is fast
```
Then:
```rust
use obsidian_parser::prelude::*;
use petgraph::dot::{Dot, Config};
let vault = Vault::open_default("/path/to/vault")?;
let graph = vault.get_digraph();
// Export to Graphviz format
println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
// Find most connected note
let most_connected = graph.node_indices()
    .max_by_key(|n| graph.edges(*n).count())
    .unwrap();
println!("Knowledge hub: {}", graph[most_connected]);
```
## Example: Analyze Knowledge Connectivity
Included example `analyzer` calculates connected components in your Obsidian vault's knowledge graph:

```bash
cargo run --example analyzer --release --features="petgraph rayon"
```
## Limitations
⚠️ **Critical Requirement for Graph Analysis**:  
All note filenames must be unique. Use `vault.has_unique_filenames()` to verify before calling `get_digraph()` or `get_ungraph()`.  
Why? Notes are identified by filename in graph operations. Duplicates will cause incorrect graph construction.
## Performance
My PC AMD Ryzen 5 3600X with NVMe SSD
| Operation               | Time       |
|-------------------------|------------|
| Vault initialization    | 741.92 µs  |
| Graph construction      | 1.67 ms    |
| Peak memory usage       | 900 KiB    |
## License
MIT © [CryptoGladi](https://github.com/CryptoGladi)
