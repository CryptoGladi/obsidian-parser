use dialoguer::Input;
use obsidian_parser::vault::Vault;
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
    let graph = vault.get_ungraph();
    println!("Time get graph: {:.2?}", get_graph.elapsed());

    println!("Count nodes in graph: {}", graph.node_count());
    println!("Count edges in graph: {}", graph.edge_count());

    println!(
        "connected components in graph: {}",
        connected_components(&graph)
    );
}
