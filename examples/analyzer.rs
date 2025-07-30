use clap::Parser;
use obsidian_parser::prelude::*;
use petgraph::algo::connected_components;
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
    let args = Args::parse();

    let open_vault = Instant::now();
    let vault = Vault::open_default(&args.path).unwrap();
    println!("Time open vault: {:.2?}", open_vault.elapsed());

    let get_graph = Instant::now();
    let ungraph = vault.get_ungraph().unwrap();
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

    let digraph = vault.get_digraph().unwrap();
    let most_connected = digraph
        .node_indices()
        .max_by_key(|n| digraph.edges(*n).count())
        .unwrap();
    println!("Knowledge hub in digraph: {}", digraph[most_connected]);
}
