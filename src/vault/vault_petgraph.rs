//! Graph analysis for Obsidian vaults using [`petgraph`](https://docs.rs/petgraph/latest/petgraph)
//!
//! This module provides functionality to convert an Obsidian vault into:
//! - **Directed graphs** ([`DiGraph`]) where edges represent one-way links
//! - **Undirected graphs** ([`UnGraph`]) where connections are bidirectional
//!
//! # Key Features
//! - Efficient graph construction using parallel processing (with `rayon` feature)
//! - Smart link parsing that handles Obsidian's link formats
//! - Memory-friendly design (prefer [`ObFileOnDisk`](crate::prelude::ObFileOnDisk) for large vaults)
//!
//! # Why [`ObFileOnDisk`](crate::prelude::ObFileOnDisk) > [`ObFileInMemory`](crate::prelude::ObFileInMemory)?
//! [`ObFileOnDisk`](crate::prelude::ObFileOnDisk) is recommended for large vaults because:
//! 1. **Lower memory usage**: Only reads file content on demand
//! 2. **Better scalability**: Avoids loading entire vault into RAM
//! 3. **Faster initialization**: Defers parsing until needed
//!
//! Use [`ObFileInMemory`](crate::prelude::ObFileInMemory) only for small vaults or when you
//! need repeated access to content.
//!
//! # Requirements
//! Enable [`petgraph`](https://docs.rs/petgraph/latest/petgraph) feature in Cargo.toml:
//! ```toml
//! [dependencies]
//! obsidian-parser = { version = "0.3", features = ["petgraph"] }
//! ```
//!
//! # Examples
//!
//! ## Basic Graph Analysis
//! ```no_run
//! use obsidian_parser::prelude::*;
//! use petgraph::dot::{Dot, Config};
//!
//! // Load vault (uses ObFileOnDisk by default)
//! let vault = Vault::open_default("/path/to/vault").unwrap();
//!
//! // Build directed graph
//! let graph = vault.get_digraph().unwrap();
//!
//! // Export to Graphviz format
//! println!("{:?}", Dot::with_config(&graph, &[Config::EdgeNoLabel]));
//! ```
//!
//! ## Advanced Connectivity Analysis
//! ```no_run
//! use obsidian_parser::prelude::*;
//! use petgraph::algo;
//!
//! let vault = Vault::open_default("/path/to/vault").unwrap();
//! let graph = vault.get_ungraph().unwrap();
//!
//! // Find knowledge clusters
//! let components = algo::connected_components(&graph);
//! println!("Found {} knowledge clusters", components);
//! ```
//!
//! ## Custom Properties with Graph Analysis
//! ```no_run
//! use obsidian_parser::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Clone)]
//! struct NoteProperties {
//!     importance: Option<usize>,
//! }
//!
//! // Load vault with custom properties
//! let vault: Vault<NoteProperties> = Vault::open("/path/to/vault").unwrap();
//!
//! // Build graph filtering by property
//! let mut graph = vault.get_digraph().unwrap();
//!
//! // Remove low-importance nodes
//! graph.retain_nodes(|g, n| {
//!     vault.files[n.index()].properties().unwrap().unwrap().importance.unwrap_or(0) > 5
//! });
//! ```

use super::Vault;
use crate::error::Error;
use crate::obfile::ObFile;
use ahash::AHashMap;
use petgraph::graph::NodeIndex;
use petgraph::{
    EdgeType, Graph,
    graph::{DiGraph, UnGraph},
};
use serde::de::DeserializeOwned;
use std::marker::{Send, Sync};

/// Parses Obsidian-style links in note content
///
/// Handles all link formats:
/// - `[[Note]]`
/// - `[[Note|Alias]]`
/// - `[[Note^block]]`
/// - `[[Note#heading]]`
/// - `[[Note#heading|Alias]]`
///
/// # Example
/// ```
/// # use obsidian_parser::vault::vault_petgraph::parse_links;
/// let content = "[[Physics]] and [[Math|Mathematics]]";
/// let links: Vec<_> = parse_links(content).collect();
/// assert_eq!(links, vec!["Physics", "Math"]);
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
pub fn parse_links(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("[[").filter_map(move |(start_pos, _)| {
        let end_pos = text[start_pos + 2..].find("]]")?;
        let inner = &text[start_pos + 2..start_pos + 2 + end_pos];

        let note_name = inner
            .split('#')
            .next()?
            .split('^')
            .next()?
            .split('|')
            .next()?
            .trim();

        Some(note_name)
    })
}

impl<T, F> Vault<T, F>
where
    T: DeserializeOwned + Clone,
    F: ObFile<T> + Send + Sync,
{
    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    #[cfg(feature = "rayon")]
    fn build_edges_for_graph<Ty: EdgeType + Send + Sync>(
        graph: &mut Graph<String, (), Ty>,
        files: &[F],
        nodes: &AHashMap<String, usize>,
    ) {
        use rayon::prelude::*;

        const CHUNK_SIZE: usize = 10;

        #[cfg(feature = "logging")]
        log::debug!("Using parallel edge builder (rayon enabled)");

        let (tx, rx) = crossbeam_channel::unbounded();

        rayon::scope(|s| {
            s.spawn(|_| {
                files
                    .into_par_iter()
                    .chunks(CHUNK_SIZE)
                    .for_each_with(tx, |tx, files| {
                        let mut result = Vec::with_capacity(10 * CHUNK_SIZE);

                        for file in files {
                            #[allow(
                                clippy::unwrap_used,
                                reason = "When creating a Vault, the path will be mandatory"
                            )]
                            let name = file.note_name().unwrap();

                            parse_links(&file.content().expect("read contect error"))
                                .filter(|link| nodes.contains_key(*link))
                                .map(|link| {
                                    let node_to = nodes[&name];
                                    let node_from = nodes[link];

                                    (node_to, node_from)
                                })
                                .for_each(|x| result.push(x));
                        }

                        #[allow(clippy::unwrap_used)]
                        tx.send(result).unwrap();
                    });
            });

            s.spawn(|_| {
                while let Ok(result) = rx.recv() {
                    for (node_to, node_from) in result {
                        graph.add_edge(NodeIndex::new(node_to), NodeIndex::new(node_from), ());
                    }
                }
            });
        });
    }

    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    #[cfg(not(feature = "rayon"))]
    fn build_edges_for_graph<Ty: EdgeType>(
        graph: &mut Graph<String, (), Ty>,
        files: &[F],
        nodes: &AHashMap<String, usize>,
    ) {
        #[cfg(feature = "logging")]
        log::debug!("Using sequential edge builder");

        for file in files {
            #[allow(
                clippy::unwrap_used,
                reason = "When creating a Vault, the path will be mandatory"
            )]
            let name = file.note_name().unwrap();

            parse_links(&file.content().expect("read contect error"))
                .filter(|link| nodes.contains_key(*link))
                .for_each(|link| {
                    let node_to = nodes[&name];
                    let node_from = nodes[link];

                    graph.add_edge(NodeIndex::new(node_to), NodeIndex::new(node_from), ());
                });
        }
    }

    /// Internal graph builder shared by both graph types
    ///
    /// # Panics
    /// Panics if duplicate note names exist.
    /// Always run [`check_unique_note_name`](Vault::check_unique_note_name) first!
    fn build_graph<Ty: EdgeType + Send + Sync>(
        &self,
        graph: &mut Graph<String, (), Ty>,
    ) -> Result<(), Error> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Building graph for vault: {} ({} files)",
            self.path.display(),
            self.files.len()
        );

        let duplicated_notes = self.get_duplicates_notes();
        if !duplicated_notes.is_empty() {
            return Err(Error::DuplicateNoteNamesDetected(duplicated_notes));
        }

        let mut nodes = AHashMap::default();
        for file in &self.files {
            #[allow(
                clippy::unwrap_used,
                reason = "When creating a Vault, the path will be mandatory"
            )]
            let name = file.note_name().unwrap();

            let node = graph.add_node(name.clone());
            nodes.insert(name, node.index());
        }

        Self::build_edges_for_graph(graph, &self.files, &nodes);

        #[cfg(feature = "logging")]
        log::debug!("Graph construction complete. Edges: {}", graph.edge_count());

        Ok(())
    }

    /// Builds directed graph representing note relationships
    ///
    /// Edges point from source note to linked note (A → B means A links to B)
    ///
    /// # Performance Notes
    /// - For vaults with 1000+ notes, enable `rayon` feature
    /// - Uses [`ObFileOnDisk`](crate::prelude::ObFileOnDisk) for minimal memory footprint
    ///
    /// # Example
    /// ```no_run
    /// # use obsidian_parser::prelude::*;
    /// # use petgraph::Direction;
    /// # let vault = Vault::open_default("test_vault").unwrap();
    /// let graph = vault.get_digraph().unwrap();
    ///
    /// // Analyze note influence
    /// let mut influence_scores: Vec<_> = graph.node_indices()
    ///     .map(|i| (i, graph.edges_directed(i, Direction::Incoming).count()))
    ///     .collect();
    ///
    /// influence_scores.sort_by_key(|(_, count)| *count);
    /// println!("Most influential note: {:?}", influence_scores.last().unwrap());
    /// ```
    ///
    /// # Errors
    /// - [`Error::DuplicateNoteNamesDetected`]
    ///
    /// # Other
    /// See [`get_ungraph`](Vault::get_ungraph)
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_digraph(&self) -> Result<DiGraph<String, ()>, Error> {
        #[cfg(feature = "logging")]
        log::debug!("Building directed graph");

        let mut graph = DiGraph::new();
        self.build_graph(&mut graph)?;

        Ok(graph)
    }

    /// Builds undirected graph showing note connections
    ///
    /// Useful for connectivity analysis where direction doesn't matter
    ///
    /// # Example
    /// ```no_run
    /// # use obsidian_parser::prelude::*;
    /// # use petgraph::algo;
    /// # let vault = Vault::open_default("test_vault").unwrap();
    /// let graph = vault.get_ungraph().unwrap();
    ///
    /// // Find connected components
    /// let components = algo::connected_components(&graph);
    /// println!("Found {} knowledge clusters", components);
    /// ```
    ///
    /// # Errors
    /// - [`Error::DuplicateNoteNamesDetected`]
    ///
    /// # Other
    /// See [`get_digraph`](Vault::get_digraph)
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_ungraph(&self) -> Result<UnGraph<String, ()>, Error> {
        #[cfg(feature = "logging")]
        log::debug!("Building undirected graph");

        let mut graph = UnGraph::new_undirected();
        self.build_graph(&mut graph)?;

        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::init_test_logger;
    use crate::vault::vault_test::create_test_vault;

    #[test]
    fn get_digraph() {
        init_test_logger();
        let (vault_path, files) = create_test_vault().unwrap();
        let vault = Vault::open_default(vault_path.path()).unwrap();

        let graph = vault.get_digraph().unwrap();
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.node_count(), files.len());
    }

    #[test]
    fn get_ungraph() {
        init_test_logger();
        let (vault_path, files) = create_test_vault().unwrap();
        let vault = Vault::open_default(vault_path.path()).unwrap();

        let graph = vault.get_ungraph().unwrap();
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.node_count(), files.len());
    }

    #[test]
    fn test_parse_links() {
        init_test_logger();
        let test_data =
            "[[Note]] [[Note|Alias]] [[Note^block]] [[Note#Heading|Alias]] [[Note^block|Alias]]";

        let ds: Vec<_> = parse_links(test_data).collect();

        assert!(ds.iter().all(|x| *x == "Note"))
    }
}
