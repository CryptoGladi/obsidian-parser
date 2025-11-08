use super::index::Index;
use crate::{
    obfile::{ObFile, parse_links},
    vault::Vault,
};
use petgraph::{EdgeType, Graph};
use std::path::Path;

pub struct GraphBuilder<'a, F, Ty>
where
    F: ObFile + Send + Sync,
    Ty: EdgeType + Send,
{
    vault: &'a Vault<F>,
    graph: Graph<String, (), Ty>,
}

impl<'a, F, Ty> GraphBuilder<'a, F, Ty>
where
    F: ObFile + Send + Sync,
    Ty: EdgeType + Send,
{
    pub(crate) const fn new(vault: &'a Vault<F>, graph: Graph<String, (), Ty>) -> Self {
        Self { vault, graph }
    }

    pub(crate) fn build(mut self) -> Graph<String, (), Ty> {
        #[cfg(feature = "logging")]
        log::debug!(
            "Building graph for vault: {} ({} files)",
            self.vault.path.display(),
            self.vault.files.len()
        );

        let index = self.create_index();
        self.create_edges(&index);

        #[cfg(feature = "logging")]
        log::debug!(
            "Graph construction complete. Edges: {}",
            self.graph.edge_count()
        );

        self.graph
    }

    /// Get relative path
    ///
    /// # How does this work?
    /// `/home/cryptogladi/obsidian` - it is `strip_prefix`
    /// `/home/cryptogladi/obsidian/file.md` - it is `file`
    ///
    /// 1. Delete `strip_prefix` from `file`: `file.md`
    /// 2. Delete `.md`: `file`
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

    fn create_index(&mut self) -> Index {
        #[cfg(feature = "logging")]
        log::debug!("Creating index...");

        let mut index = Index::default();

        #[allow(
            clippy::unwrap_used,
            reason = "When creating a Vault, the path will be mandatory"
        )]
        for file in &self.vault.files {
            let full = Self::relative_path(file, &self.vault.path);
            let short = file.note_name().unwrap();

            let node = self.graph.add_node(full.clone());
            index.insert(full, short, node);
        }

        #[cfg(feature = "logging")]
        log::debug!("Done create index for {} files", self.vault.files.len());

        index
    }

    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    #[cfg(feature = "rayon")]
    fn create_edges(&mut self, index: &Index) {
        use rayon::prelude::*;

        const CHUNK_SIZE: usize = 10;

        #[cfg(feature = "logging")]
        log::debug!("Using parallel edge builder (rayon enabled)");

        let (tx, rx) = crossbeam_channel::unbounded();
        let files = &self.vault.files;
        let strip_prefix = &self.vault.path;
        let graph = &mut self.graph;

        rayon::scope(|s| {
            s.spawn(|_| {
                files
                    .into_par_iter()
                    .chunks(CHUNK_SIZE)
                    .for_each_with(tx, |tx, files| {
                        let mut result = Vec::with_capacity(10 * CHUNK_SIZE);

                        for file in files {
                            let path = Self::relative_path(file, strip_prefix);
                            let node_to = index.full[&path];

                            parse_links(&file.content().expect("read content"))
                                .filter_map(|link| index.get(link))
                                .map(|node_from| (node_to, *node_from))
                                .for_each(|x| result.push(x));
                        }

                        #[allow(clippy::unwrap_used)]
                        tx.send(result).unwrap();
                    });
            });

            s.spawn(|_| {
                while let Ok(result) = rx.recv() {
                    for (node_to, node_from) in result {
                        graph.add_edge(node_to, node_from, ());
                    }
                }
            });
        });
    }

    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    #[cfg(not(feature = "rayon"))]
    fn create_edges(&mut self, index: &Index) {
        #[cfg(feature = "logging")]
        log::debug!("Using sequential edge builder");

        for file in &self.vault.files {
            let path = Self::relative_path(file, &self.vault.path);
            let node_to = index.full[&path];

            parse_links(&file.content().expect("read content"))
                .filter_map(|link| index.get(link))
                .map(|node_from| (node_to, *node_from))
                .for_each(|(node_to, node_from)| {
                    self.graph.add_edge(node_to, node_from, ());
                });
        }
    }
}
