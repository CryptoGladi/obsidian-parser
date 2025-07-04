use criterion::{Criterion, criterion_group, criterion_main};
use obsidian_parser::vault::Vault;
use rand::Rng;
use serde::Deserialize;
use std::hint::black_box;
use std::io::Write;
use tempfile::TempDir;

#[derive(Default, Debug, Clone, Deserialize)]
struct NoteProperties {
    id: usize,
}

fn prepare_test_file(
    file: &mut std::fs::File,
    num_files: usize,
    links_per_file: usize,
    properties: &NoteProperties,
) {
    let mut rng = rand::rng();

    // Frontmatter
    writeln!(file, "---\nid: {}\n---", properties.id).unwrap();

    for _ in 0..links_per_file {
        if rng.random::<bool>() {
            writeln!(file).unwrap(); // Empty lines for realism
        }

        for _ in 0..rng.random_range(50..=100) {
            writeln!(file, "TEST DATA").unwrap();
        }

        // Random target for link
        let target = rng.random_range(0..num_files);
        writeln!(file, "Link [[note_{}]]", target).unwrap();
    }
}

fn generate_test_vault(num_files: usize, links_per_file: usize) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path();

    for i in 0..num_files {
        let file_path = dir_path.join(format!("note_{}.md", i));
        prepare_test_file(
            &mut std::fs::File::create(file_path).unwrap(),
            num_files,
            links_per_file,
            &NoteProperties { id: i },
        );
    }

    temp_dir
}

fn vault_open_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    c.bench_function("vault_open", |b| {
        b.iter(|| {
            let vault: Vault<NoteProperties> = Vault::open(black_box(path)).unwrap();
            black_box(vault);
        })
    });
}

fn digraph_build_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();
    let vault: Vault<NoteProperties> = Vault::open(path).unwrap();

    c.bench_function("digraph_build", |b| {
        b.iter(|| {
            #[cfg(feature = "petgraph")]
            let graph = vault.get_digraph();
            black_box(graph);
        })
    });
}

fn ungraph_build_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();
    let vault = Vault::open_default(path).unwrap();

    c.bench_function("ungraph_build", |b| {
        b.iter(|| {
            #[cfg(feature = "petgraph")]
            let graph = vault.get_ungraph();
            black_box(graph);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_secs(1));
    targets = vault_open_benchmark, digraph_build_benchmark, ungraph_build_benchmark
}

criterion_main!(benches);
