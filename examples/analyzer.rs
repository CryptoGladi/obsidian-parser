use dialoguer::Input;
use obsidian_parser::vault::Vault;
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
    //let path = "/home/gladi/Obsidian";

    let open_vault = Instant::now();
    let vault = Vault::open_default(&path).unwrap();
    println!("Time open vault: {:.2?}", open_vault.elapsed());

    //println!("Same check done? {}", vault.same_name_check());

    let get_graph = Instant::now();
    let graph = vault.get_ungraph();
    println!("Time get graph: {:.2?}", get_graph.elapsed());
    println!("Count nodes in graph: {}", graph.node_count());
    println!("Count edges in graph: {}", graph.edge_count());
    /*

    let n = graph.node_count() as f64;
    let m = graph.edge_count() as f64;
    let density = 2.0 * m / (n * (n - 1.0));
    println!("Density: {}", density);

    println!(
        "connected components in graph: {}",
        connected_components(&graph)
    );

    let cliques = maximal_cliques(&graph);
    println!("count cliques: {}", cliques.len());
    println!(
        "max cliques: {}",
        cliques.iter().max_by_key(|x| x.len()).unwrap().len()
    );

    let scc = kosaraju_scc(&graph);
    let giant_component = scc.iter().max_by_key(|c| c.len()).unwrap();
    let giant_size = giant_component.len();
    println!("scc count: {}", scc.len());
    println!("giant size: {}", giant_size);

    let articulation_points = articulation_points(&graph);
    println!("articulation_points: {}", articulation_points.len()); */
}
