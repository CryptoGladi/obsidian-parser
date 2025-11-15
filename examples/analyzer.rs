use clap::Parser;
use obsidian_parser::{prelude::*, vault::vault_open::FilesBuilder};
use petgraph::algo::connected_components;
use rayon::prelude::*;
use std::{path::PathBuf, time::Instant};

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
    env_logger::init();
    let args = Args::parse();

    let open_vault = Instant::now();
    let options = VaultOptions::new(&args.path);
    let files = FilesBuilder::new(&options)
        .include_hidden(false)
        .into_par_iter()
        .filter_map(|file| match file {
            Ok(file) => Some(file),
            Err(error) => {
                eprintln!("Parsed error: {}", error);
                None
            }
        });

    let vault: VaultOnDisk = files.build_vault(&options).unwrap();
    println!("Time open vault: {:.2?}", open_vault.elapsed());
    println!("Count notes: {}", vault.count_notes());

    println!(
        "Check unique note name: {}",
        vault.have_duplicates_notes_by_name()
    );

    /* TODO

    let vault = Vault::open_default(&args.path).unwrap();

    let get_graph = Instant::now();
    let ungraph = vault.get_ungraph();
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
    println!("Knowledge hub in ungraph: {}", ungraph[most_connected]);

    let digraph = vault.get_digraph();
    let most_connected = digraph
        .node_indices()
        .max_by_key(|n| digraph.edges(*n).count())
        .unwrap();
    println!("Knowledge hub in digraph: {}", digraph[most_connected]);
    */
}
