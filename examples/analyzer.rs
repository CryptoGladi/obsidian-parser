use clap::Parser;
use obsidian_parser::{prelude::*, vault::vault_open::VaultBuilder};
use petgraph::algo::connected_components;
use rayon::prelude::*;
use sha2::Sha256;
use std::{path::PathBuf, time::Instant};
use tracing_subscriber::EnvFilter;

fn parse_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);

    if !path.is_dir() {
        return Err(format!("{} is not dir", path.display()));
    }

    Ok(path)
}

#[derive(Parser, Debug)]
struct Args {
    /// Name of the person to greet
    #[arg(long, value_parser = parse_path)]
    path: PathBuf,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let open_vault = Instant::now();
    let options = VaultOptions::new(&args.path);
    let files = VaultBuilder::new(&options)
        .include_hidden(false)
        .into_par_iter::<NoteOnceLock>()
        .filter_map(|note| match note {
            Ok(note) => Some(note),
            Err(error) => {
                eprintln!("Parsed error: {}", error);
                None
            }
        })
        .filter(|note| {
            let content = note.content().unwrap();
            !content.is_empty()
        })
        .filter(|note| !note.is_todo().unwrap());

    let vault: VaultOnceLock = files.build_vault(&options);
    println!("Time open vault: {:.2?}", open_vault.elapsed());
    println!("Count notes: {}", vault.count_notes());

    println!(
        "Check unique note name by name: {}",
        !vault.have_duplicates_notes_by_name()
    );

    println!(
        "Check unique note name by content: {}",
        !vault.have_duplicates_notes_by_content::<Sha256>().unwrap()
    );

    let word_count: usize = vault
        .notes()
        .iter()
        .map(|note| {
            note.content()
                .unwrap_or_default()
                .split_whitespace()
                .count()
        })
        .sum();
    println!("Word count: {word_count}");

    let get_graph = Instant::now();
    let ungraph = vault.par_get_ungraph().unwrap();

    println!("Time get graph: {:.2?}", get_graph.elapsed());

    println!("Count nodes in graph: {}", ungraph.node_count());
    println!("Count edges in graph: {}", ungraph.edge_count());

    println!(
        "connected components in graph: {}",
        connected_components(&ungraph)
    );

    // Find most connected note
    let most_connected = ungraph
        .node_indices()
        .max_by_key(|n| ungraph.edges(*n).count())
        .unwrap();
    println!("Knowledge hub in ungraph: {:?}", ungraph[most_connected]);
}
