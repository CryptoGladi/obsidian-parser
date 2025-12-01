use super::index::Index;
use crate::note::parser::parse_links;
use crate::{note::Note, vault::Vault};
use petgraph::{EdgeType, Graph};
use std::path::Path;

pub struct GraphBuilder<'a, F>
where
    F: Note,
{
    vault: &'a Vault<F>,
}

impl<'a, F> GraphBuilder<'a, F>
where
    F: Note,
{
    pub(crate) const fn new(vault: &'a Vault<F>) -> Self {
        Self { vault }
    }

    pub(crate) fn build<Ty>(self) -> Result<Graph<&'a F, (), Ty>, F::Error>
    where
        Ty: EdgeType,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Building graph for vault: {} ({} notes)",
            self.vault.path.display(),
            self.vault.count_notes()
        );

        let (index, mut graph) = self.create_index_with_graph();
        self.create_edges(&index, &mut graph)?;

        #[cfg(feature = "tracing")]
        tracing::debug!("Graph construction complete. Edges: {}", graph.edge_count());

        Ok(graph)
    }

    #[cfg(feature = "rayon")]
    pub(crate) fn par_build<Ty>(self) -> Result<Graph<&'a F, (), Ty>, F::Error>
    where
        F: Send + Sync,
        F::Error: Send,
        Ty: EdgeType + Send,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!(
            "Building graph for vault: {} ({} notes)",
            self.vault.path.display(),
            self.vault.count_notes()
        );

        let (index, mut graph) = self.create_index_with_graph();
        self.par_create_edges(&index, &mut graph)?;

        #[cfg(feature = "tracing")]
        tracing::debug!("Graph construction complete. Edges: {}", graph.edge_count());

        Ok(graph)
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

    fn create_index_with_graph<Ty>(&self) -> (Index, Graph<&'a F, (), Ty>)
    where
        Ty: EdgeType,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!("Creating index...");

        let mut graph = Graph::default();
        let mut index = Index::default();

        #[allow(
            clippy::unwrap_used,
            reason = "When creating a Vault, the path will be mandatory"
        )]
        for note in self.vault.notes() {
            let full = Self::relative_path(note, &self.vault.path);
            let short = note.note_name().unwrap();

            let node = graph.add_node(note);
            index.insert(full, short, node);
        }

        #[cfg(feature = "tracing")]
        tracing::debug!("Done create index for {} notes", self.vault.count_notes());

        (index, graph)
    }

    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    #[cfg(feature = "rayon")]
    fn par_create_edges<Ty>(
        &self,
        index: &Index,
        graph: &mut Graph<&'a F, (), Ty>,
    ) -> Result<(), F::Error>
    where
        F: Send + Sync,
        F::Error: Send,
        Ty: EdgeType + Send,
    {
        use petgraph::graph::NodeIndex;
        use rayon::prelude::*;

        const CHUNK_SIZE: usize = 10;

        #[cfg(feature = "tracing")]
        tracing::debug!("Using parallel edge builder (rayon enabled)");

        #[allow(clippy::items_after_statements)]
        enum Data<'a, E: Send> {
            Successful(Vec<(&'a NodeIndex, NodeIndex)>),
            Error(E),
        }

        let (tx, rx) = crossbeam_channel::unbounded();
        let notes = &self.vault.notes();
        let strip_prefix = &self.vault.path;
        let mut result = Ok(());

        rayon::scope(|s| {
            s.spawn(|_| {
                notes
                    .into_par_iter()
                    .chunks(CHUNK_SIZE)
                    .for_each_with(tx, |tx, notes| {
                        let mut result = Vec::with_capacity(10 * CHUNK_SIZE);

                        for note in notes {
                            let path = Self::relative_path(note, strip_prefix);

                            if let Some(node_to) = index.full(&path) {
                                match note.content() {
                                    Ok(content) => parse_links(&content)
                                        .filter_map(|link| index.get(link))
                                        .map(|node_from| (node_to, *node_from))
                                        .for_each(|x| result.push(x)),
                                    Err(error) => tx.send(Data::Error(error)).expect("Send error"),
                                }
                            }
                        }

                        #[allow(clippy::unwrap_used)]
                        tx.send(Data::Successful(result)).unwrap();
                    });
            });

            s.spawn(|_| {
                while let Ok(recv) = rx.recv() {
                    match recv {
                        Data::Successful(notes) => {
                            for (note_to, note_from) in notes {
                                graph.add_edge(*note_to, note_from, ());
                            }
                        }
                        Data::Error(error) => result = Err(error),
                    }
                }
            });
        });

        result
    }

    /// Builds edges between nodes in the graph
    ///
    /// Uses parallel processing when `rayon` feature is enabled
    fn create_edges<Ty>(
        &self,
        index: &Index,
        graph: &mut Graph<&'a F, (), Ty>,
    ) -> Result<(), F::Error>
    where
        Ty: EdgeType,
    {
        #[cfg(feature = "tracing")]
        tracing::debug!("Using sequential edge builder");

        for file in self.vault.notes() {
            let path = Self::relative_path(file, &self.vault.path);

            if let Some(node_to) = index.full(&path) {
                let content = file.content()?;

                parse_links(&content)
                    .filter_map(|link| index.get(link))
                    .map(|node_from| (node_to, *node_from))
                    .for_each(|(node_to, node_from)| {
                        graph.add_edge(*node_to, node_from, ());
                    });
            }
        }

        Ok(())
    }
}
