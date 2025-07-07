use dialoguer::Input;
use obsidian_parser::prelude::*;
use petgraph::algo::connected_components;
use std::{
    path::{Path, PathBuf},
    time::Instant,
};

fn get_path() -> PathBuf {
    let path = Input::new()
        .with_prompt("Path to Obsidian vault")
        .validate_with(|x: &String| {
            if Path::new(x).is_dir() {
                return Ok(());
            }

            Err("Is not dir")
        })
        .interact_text()
        .unwrap();

    PathBuf::from(path)
}

fn main() {
    let path = get_path();

    let open_vault = Instant::now();
    let vault = Vault::open_default(&path).unwrap();
    println!("Time open vault: {:.2?}", open_vault.elapsed());

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
}
