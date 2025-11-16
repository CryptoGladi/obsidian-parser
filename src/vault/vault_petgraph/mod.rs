//! Graph analysis for Obsidian vaults using [`petgraph`](https://docs.rs/petgraph/latest/petgraph)
//!
//! This module provides functionality to convert an Obsidian vault into:
//! - **Directed graphs** ([`DiGraph`]) where edges represent one-way links
//! - **Undirected graphs** ([`UnGraph`]) where connections are bidirectional
//!
//! # Key Features
//! - Efficient graph construction using parallel processing (with `rayon` feature)
//! - Smart link parsing that handles Obsidian's link formats
//! - Memory-friendly design (prefer [`NoteOnDisk`](crate::prelude::NoteOnDisk) for large vaults)
//!
//! # Why [`NoteOnDisk`](crate::prelude::NoteOnDisk) > [`NoteInMemory`](crate::prelude::NoteInMemory)?
//! [`NoteOnDisk`](crate::prelude::NoteOnDisk) is recommended for large vaults because:
//! 1. **Lower memory usage**: Only reads file content on demand
//! 2. **Better scalability**: Avoids loading entire vault into RAM
//! 3. **Faster initialization**: Defers parsing until needed
//!
//! Use [`NoteInMemory`](crate::prelude::NoteInMemory) only for small vaults or when you
//! need repeated access to content.
//!
//! # Requirements
//! Enable [`petgraph`](https://docs.rs/petgraph/latest/petgraph) feature in Cargo.toml:
//! ```toml
//! [dependencies]
//! obsidian-parser = { version = "0.6", features = ["petgraph"] }
//! ```

mod graph_builder;
mod index;

use super::Vault;
use crate::note::Note;
use graph_builder::GraphBuilder;
use petgraph::{
    EdgeType, Graph,
    graph::{DiGraph, UnGraph},
};
use std::marker::{Send, Sync};

impl<F> Vault<F>
where
    F: Note,
{
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_graph<Ty>(&self) -> Result<Graph<&F, (), Ty>, F::Error>
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
    pub fn par_get_graph<Ty>(&self) -> Result<Graph<&F, (), Ty>, F::Error>
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
    /// - Uses [`NoteOnDisk`](crate::prelude::NoteOnDisk) for minimal memory footprint
    ///
    /// # Other
    /// See [`get_ungraph`](Vault::get_ungraph)
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_digraph(&self) -> Result<DiGraph<&F, ()>, F::Error> {
        #[cfg(feature = "logging")]
        log::debug!("Building directed graph");

        self.get_graph()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    #[cfg(feature = "rayon")]
    pub fn par_get_digraph(&self) -> Result<DiGraph<&F, ()>, F::Error>
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
    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    pub fn get_ungraph(&self) -> Result<UnGraph<&F, ()>, F::Error> {
        #[cfg(feature = "logging")]
        log::debug!("Building undirected graph");

        self.get_graph()
    }

    #[cfg_attr(docsrs, doc(cfg(feature = "petgraph")))]
    #[cfg(feature = "rayon")]
    pub fn par_get_ungraph(&self) -> Result<UnGraph<&F, ()>, F::Error>
    where
        F: Send + Sync,
        F::Error: Send,
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
