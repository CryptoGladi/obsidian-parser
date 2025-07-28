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
}
