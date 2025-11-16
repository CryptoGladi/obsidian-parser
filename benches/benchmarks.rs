use criterion::{Criterion, criterion_group, criterion_main};
use obsidian_parser::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::io::Write;
use tempfile::TempDir;

#[derive(Default, Debug, Clone, Deserialize, Serialize)]
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
            let options = VaultOptions::new(black_box(path));
            let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_iter()
                .map(|note| note.unwrap())
                .build_vault(&options)
                .unwrap();

            black_box(vault);
        })
    });

    #[cfg(feature = "rayon")]
    c.bench_function("parallel vault_open_on_disk", |b| {
        b.iter(|| {
            use rayon::prelude::*;

            let options = VaultOptions::new(black_box(path));
            let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_par_iter()
                .map(|note| note.unwrap())
                .build_vault(&options)
                .unwrap();

            black_box(vault);
        })
    });

    c.bench_function("vault_open_in_memory", |b| {
        b.iter(|| {
            let options = VaultOptions::new(black_box(path));
            let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_iter()
                .map(|note| note.unwrap())
                .build_vault(&options)
                .unwrap();

            black_box(vault);
        })
    });

    #[cfg(feature = "rayon")]
    c.bench_function("parallel vault_open_in_memory", |b| {
        b.iter(|| {
            use rayon::prelude::*;

            let options = VaultOptions::new(black_box(path));
            let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_par_iter()
                .map(|note| note.unwrap())
                .build_vault(&options)
                .unwrap();

            black_box(vault);
        })
    });
}

#[cfg(feature = "petgraph")]
fn graph_build_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_on_disk = {
        let options = VaultOptions::new(path);
        let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|note| note.unwrap())
            .build_vault(&options)
            .unwrap();

        vault
    };

    c.bench_function("graph_build_on_disk", |b| {
        b.iter(|| {
            let graph = vault_on_disk.get_digraph().unwrap();
            black_box(graph);
        })
    });

    #[cfg(feature = "rayon")]
    c.bench_function("parallel graph_build_on_disk", |b| {
        b.iter(|| {
            let graph = vault_on_disk.par_get_digraph().unwrap();
            black_box(graph);
        })
    });

    let vault_in_memory = {
        let options = VaultOptions::new(path);
        let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|note| note.unwrap())
            .build_vault(&options)
            .unwrap();

        vault
    };

    c.bench_function("graph_build_in_memory", |b| {
        b.iter(|| {
            let graph = vault_in_memory.get_digraph().unwrap();
            black_box(graph);
        })
    });

    #[cfg(feature = "rayon")]
    c.bench_function("parallel graph_build_in_memory", |b| {
        b.iter(|| {
            let graph = vault_in_memory.par_get_digraph().unwrap();
            black_box(graph);
        })
    });
}

fn get_duplicates_by_name_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_on_disk = {
        let options = VaultOptions::new(path);
        let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|note| note.unwrap())
            .build_vault(&options)
            .unwrap();

        vault
    };

    let vault_in_memory = {
        let options = VaultOptions::new(path);
        let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|note| note.unwrap())
            .build_vault(&options)
            .unwrap();

        vault
    };

    c.bench_function("get_duplicates_notes_by_name (VaultOnDisk)", |b| {
        b.iter(|| {
            let duplicated = vault_on_disk.get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });

    #[cfg(feature = "rayon")]
    c.bench_function("parallel get_duplicates_notes_by_name (VaultOnDisk)", |b| {
        b.iter(|| {
            let duplicated = vault_on_disk.par_get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });

    c.bench_function("get_duplicates_notes_by_name (VaultInMemory)", |b| {
        b.iter(|| {
            let duplicated = vault_in_memory.get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });

    #[cfg(feature = "rayon")]
    c.bench_function(
        "parallel get_duplicates_notes_by_name (VaultInMemory)",
        |b| {
            b.iter(|| {
                let duplicated = vault_in_memory.par_get_duplicates_notes_by_name();
                black_box(duplicated);
            })
        },
    );
}

#[cfg(feature = "digest")]
fn get_duplicates_by_content_benchmark(c: &mut Criterion) {
    use sha2::Sha256;

    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_on_disk = {
        let options = VaultOptions::new(path);
        let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|note| note.unwrap())
            .build_vault(&options)
            .unwrap();

        vault
    };

    let vault_in_memory = {
        let options = VaultOptions::new(path);
        let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
            .include_hidden(true)
            .into_iter()
            .map(|note| note.unwrap())
            .build_vault(&options)
            .unwrap();

        vault
    };

    c.bench_function(
        "get_duplicates_notes_by_content with Sha256 (VaultOnDisk)",
        |b| {
            b.iter(|| {
                let duplicated = vault_on_disk
                    .get_duplicates_notes_by_content::<Sha256>()
                    .unwrap();

                black_box(duplicated);
            })
        },
    );

    c.bench_function(
        "get_duplicates_notes_by_content with Sha256 (VaultOnDisk)",
        |b| {
            b.iter(|| {
                let duplicated = vault_in_memory
                    .get_duplicates_notes_by_content::<Sha256>()
                    .unwrap();

                black_box(duplicated);
            })
        },
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(std::time::Duration::from_secs(1));
    targets = vault_open_benchmark, graph_build_benchmark, get_duplicates_by_name_benchmark, get_duplicates_by_content_benchmark
}

criterion_main!(benches);
