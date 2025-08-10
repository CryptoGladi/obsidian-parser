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
//! obsidian-parser = { version = "0.4", features = ["petgraph"] }
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
//! let graph = vault.get_digraph();
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
//! let graph = vault.get_ungraph();
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
//! let mut graph = vault.get_digraph();
//!
//! // Remove low-importance nodes
//! graph.retain_nodes(|g, n| {
//!     vault.files[n.index()].properties().unwrap().unwrap().importance.unwrap_or(0) > 5
//! });
//! ```

use super::Vault;
use crate::obfile::ObFile;
use crate::obfile::parse_links;
use ahash::AHashMap;
use petgraph::graph::NodeIndex;
use petgraph::{
    EdgeType, Graph,
    graph::{DiGraph, UnGraph},
};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::marker::{Send, Sync};
use std::path::Path;

impl<T, F> Vault<T, F>
where
    T: DeserializeOwned + Clone,
    F: ObFile<T> + Send + Sync,
{
    #[allow(
        clippy::unwrap_used,
        reason = "When creating a Vault, the path will be mandatory"
    )]
    #[inline]
    fn relative_path(file: &F, strip_prefix: &Path) -> String {
        file.path()
            .unwrap()
            .strip_prefix(strip_prefix)
            .unwrap()
            .with_extension("")
            .to_string_lossy()
            .to_string()
    }

    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    #[cfg(feature = "rayon")]
    fn build_edges_for_graph<Ty: EdgeType + Send + Sync>(
        graph: &mut Graph<String, (), Ty>,
        files: &[F],
        nodes: &AHashMap<String, usize>,
        strip_prefix: &Path,
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
                            let path = Self::relative_path(file, strip_prefix);

                            parse_links(&file.content().expect("read contect error"))
                                .filter(|link| nodes.contains_key(*link))
                                .map(|link| {
                                    let node_to = nodes.g;
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
        strip_prefix: &Path,
    ) {
        #[cfg(feature = "logging")]
        log::debug!("Using sequential edge builder");

        for file in files {
            let name = Self::relative_path(file, strip_prefix);

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
    fn build_graph<Ty: EdgeType + Send + Sync>(&self, graph: &mut Graph<String, (), Ty>) {
        #[cfg(feature = "logging")]
        log::debug!(
            "Building graph for vault: {} ({} files)",
            self.path.display(),
            self.files.len()
        );

        let mut nodes = AHashMap::default();
        for file in &self.files {
            #[allow(
                clippy::unwrap_used,
                reason = "When creating a Vault, the path will be mandatory"
            )]
            let path_to_note = Self::relative_path(file, &self.path);

            let node = graph.add_node(path_to_note.clone());
            nodes.insert(path_to_note, node.index());
        }

        Self::build_edges_for_graph(graph, &self.files, &nodes, &self.path);

        #[cfg(feature = "logging")]
        log::debug!("Graph construction complete. Edges: {}", graph.edge_count());
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
    /// let graph = vault.get_digraph();
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
    #[must_use]
    pub fn get_digraph(&self) -> DiGraph<String, ()> {
        #[cfg(feature = "logging")]
        log::debug!("Building directed graph");

        let mut graph = DiGraph::new();
        self.build_graph(&mut graph);

        graph
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
    /// let graph = vault.get_ungraph();
    ///
    /// // Find connected components
    /// let components = algo::connected_components(&graph);
    /// println!("Found {} knowledge clusters", components);
    /// ```
    ///
    /// # Other
    /// See [`get_digraph`](Vault::get_digraph)
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    #[must_use]
    pub fn get_ungraph(&self) -> UnGraph<String, ()> {
        #[cfg(feature = "logging")]
        log::debug!("Building undirected graph");

        let mut graph = UnGraph::new_undirected();
        self.build_graph(&mut graph);

        graph
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

        let graph = vault.get_digraph();
        assert_eq!(graph.edge_count(), 1);
        assert_eq!(graph.node_count(), files.len());
    }

    #[test]
    fn get_ungraph() {
        init_test_logger();
        let (vault_path, files) = create_test_vault().unwrap();
        let vault = Vault::open_default(vault_path.path()).unwrap();

        let graph = vault.get_ungraph();
        assert_eq!(graph.edge_count(), 2);
        assert_eq!(graph.node_count(), files.len());
    }
}
