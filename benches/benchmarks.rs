use criterion::{Criterion, criterion_group, criterion_main};
use obsidian_parser::prelude::*;
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

    c.bench_function("vault_open_on_disk", |b| {
        b.iter(|| {
            let vault: Vault<NoteProperties, ObFileOnDisk<NoteProperties>> =
                Vault::open(black_box(path)).unwrap();
            black_box(vault);
        })
    });

    c.bench_function("vault_open_unchecked_on_disk", |b| {
        b.iter(|| {
            let vault: Vault<NoteProperties, ObFileOnDisk<NoteProperties>> =
                unsafe { Vault::open_unchecked(black_box(path)).unwrap() };
            black_box(vault);
        })
    });

    c.bench_function("vault_open_in_memory", |b| {
        b.iter(|| {
            let vault: Vault<NoteProperties, ObFileInMemory<NoteProperties>> =
                Vault::open(black_box(path)).unwrap();
            black_box(vault);
        })
    });

    c.bench_function("vault_open_unchecked_in_memory", |b| {
        b.iter(|| {
            let vault: Vault<NoteProperties, ObFileInMemory<NoteProperties>> =
                unsafe { Vault::open_unchecked(black_box(path)).unwrap() };
            black_box(vault);
        })
    });
}

fn graph_build_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_on_disk: Vault<NoteProperties, ObFileOnDisk<NoteProperties>> =
        Vault::open(black_box(path)).unwrap();
    c.bench_function("graph_build_on_disk", |b| {
        b.iter(|| {
            let graph = vault_on_disk.get_digraph();
            black_box(graph);
        })
    });

    let vault_in_memory: Vault<NoteProperties, ObFileInMemory<NoteProperties>> =
        Vault::open(black_box(path)).unwrap();
    c.bench_function("graph_build_in_memory", |b| {
        b.iter(|| {
            let graph = vault_in_memory.get_digraph();
            black_box(graph);
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_secs(1));
    targets = vault_open_benchmark, graph_build_benchmark
}

criterion_main!(benches);
