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
//! obsidian-parser = { version = "0.6", features = ["petgraph"] }
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
//! let vault: VaultInMemory<NoteProperties> = Vault::open("/path/to/vault").unwrap();
//!
//! // Build graph filtering by property
//! let mut graph = vault.get_digraph();
//!
//! // Remove low-importance nodes
//! graph.retain_nodes(|g, n| {
//!     vault.files[n.index()].properties().unwrap().unwrap().importance.unwrap_or(0) > 5
//! });
//! ```

mod graph_builder;
mod index;

use super::Vault;
use crate::obfile::ObFile;
use graph_builder::GraphBuilder;
use petgraph::{
    EdgeType, Graph,
    graph::{DiGraph, UnGraph},
};
use std::marker::{Send, Sync};

impl<F> Vault<F>
where
    F: ObFile,
{
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_graph<'a, Ty>(&'a self) -> Result<Graph<&'a F, (), Ty>, F::Error>
    where
        Ty: EdgeType,
    {
        #[cfg(feature = "logging")]
        log::debug!("Building graph");

        let graph_builder = GraphBuilder::new(self);
        graph_builder.build()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    #[cfg(feature = "rayon")]
    pub fn par_get_graph<'a, Ty>(&'a self) -> Result<Graph<&'a F, (), Ty>, F::Error>
    where
        F: Send + Sync,
        F::Error: Send,
        Ty: EdgeType + Send,
    {
        #[cfg(feature = "logging")]
        log::debug!("Building graph with parallel");

        let graph_builder = GraphBuilder::new(self);
        graph_builder.par_build()
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
    /// # Other
    /// See [`get_ungraph`](Vault::get_ungraph)
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_digraph<'a>(&'a self) -> Result<DiGraph<&'a F, ()>, F::Error> {
        #[cfg(feature = "logging")]
        log::debug!("Building directed graph");

        self.get_graph()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    #[cfg(feature = "rayon")]
    #[must_use]
    pub fn par_get_digraph<'a>(&'a self) -> Result<DiGraph<&'a F, ()>, F::Error>
    where
        F: Send + Sync,
        F::Error: Send,
    {
        #[cfg(feature = "logging")]
        log::debug!("Building directed graph");

        self.par_get_graph()
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
    pub fn get_ungraph<'a>(&'a self) -> Result<UnGraph<&'a F, ()>, F::Error> {
        #[cfg(feature = "logging")]
        log::debug!("Building undirected graph");

        self.get_graph()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    #[cfg(feature = "rayon")]
    #[must_use]
    pub fn par_get_ungraph<'a>(&'a self) -> Result<UnGraph<&'a F, ()>, F::Error>
    where
        F: Send + Sync,
        F::Error: Send,
        F: Send + Sync,
    {
        #[cfg(feature = "logging")]
        log::debug!("Building undirected graph");

        self.par_get_graph()
    }
}

#[cfg(test)]
mod tests {
    use crate::vault::vault_test::create_test_vault;

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "petgraph")]
    fn get_digraph() {
        let (vault, _temp_dir, files) = create_test_vault().unwrap();

        let graph = vault.get_digraph().unwrap();

        assert_eq!(graph.edge_count(), 3);
        assert_eq!(graph.node_count(), files.len());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "petgraph")]
    #[cfg(feature = "rayon")]
    fn par_get_digraph() {
        let (vault, _temp_dir, files) = create_test_vault().unwrap();

        let graph = vault.par_get_digraph().unwrap();

        assert_eq!(graph.edge_count(), 3);
        assert_eq!(graph.node_count(), files.len());
    }

    #[cfg_attr(feature = "logging", test_log::test)]
    #[cfg_attr(not(feature = "logging"), test)]
    #[cfg(feature = "petgraph")]
    #[cfg(feature = "rayon")]
    fn par_get_ungraph() {
        let (vault, _temp_dir, files) = create_test_vault().unwrap();

        let graph = vault.par_get_ungraph().unwrap();
        assert_eq!(graph.edge_count(), 3);
        assert_eq!(graph.node_count(), files.len());
    }
}
