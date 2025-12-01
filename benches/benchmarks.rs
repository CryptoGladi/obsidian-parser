use criterion::{Criterion, criterion_group, criterion_main};
use obsidian_parser::prelude::*;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use std::io::Write;
use std::path::Path;
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

fn create_vault<V: NoteFromFile<Properties = NoteProperties>>(path: &Path) -> Vault<V>
where
    V::Error: From<std::io::Error>,
{
    let options = VaultOptions::new(path);
    let vault = VaultBuilder::new(&options)
        .include_hidden(true)
        .into_iter()
        .map(|note| note.unwrap())
        .build_vault(&options);

    vault
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

    c.bench_function("vault_open (VaultInMemory)", |b| {
        b.iter(|| {
            let options = VaultOptions::new(black_box(path));
            let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_iter()
                .map(|note| note.unwrap())
                .build_vault(&options);

            black_box(vault);
        })
    });

    c.bench_function("vault_open (VaultOnDisk)", |b| {
        b.iter(|| {
            let options = VaultOptions::new(black_box(path));
            let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_iter()
                .map(|note| note.unwrap())
                .build_vault(&options);

            black_box(vault);
        })
    });

    c.bench_function("vault_open (VaultOnceCell)", |b| {
        b.iter(|| {
            let options = VaultOptions::new(black_box(path));
            let vault: VaultOnceCell<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_iter()
                .map(|note| note.unwrap())
                .build_vault(&options);

            black_box(vault);
        })
    });

    c.bench_function("vault_open (VaultOnceLock)", |b| {
        b.iter(|| {
            let options = VaultOptions::new(black_box(path));
            let vault: VaultOnceLock<NoteProperties> = VaultBuilder::new(&options)
                .include_hidden(true)
                .into_iter()
                .map(|note| note.unwrap())
                .build_vault(&options);

            black_box(vault);
        })
    });

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;

        c.bench_function("Parallel vault_open (VaultInMemory)", |b| {
            b.iter(|| {
                let options = VaultOptions::new(black_box(path));
                let vault: VaultInMemory<NoteProperties> = VaultBuilder::new(&options)
                    .include_hidden(true)
                    .into_par_iter()
                    .map(|note| note.unwrap())
                    .build_vault(&options);

                black_box(vault);
            })
        });

        c.bench_function("Parallel vault_open (VaultOnDisk)", |b| {
            b.iter(|| {
                let options = VaultOptions::new(black_box(path));
                let vault: VaultOnDisk<NoteProperties> = VaultBuilder::new(&options)
                    .include_hidden(true)
                    .into_par_iter()
                    .map(|note| note.unwrap())
                    .build_vault(&options);

                black_box(vault);
            })
        });

        c.bench_function("Parallel vault_open (VaultOnceCell)", |b| {
            b.iter(|| {
                let options = VaultOptions::new(black_box(path));
                let vault: VaultOnceCell<NoteProperties> = VaultBuilder::new(&options)
                    .include_hidden(true)
                    .into_par_iter()
                    .map(|note| note.unwrap())
                    .build_vault(&options);

                black_box(vault);
            })
        });

        c.bench_function("Parallel vault_open (VaultOnceLock)", |b| {
            b.iter(|| {
                let options = VaultOptions::new(black_box(path));
                let vault: VaultOnceLock<NoteProperties> = VaultBuilder::new(&options)
                    .include_hidden(true)
                    .into_par_iter()
                    .map(|note| note.unwrap())
                    .build_vault(&options);

                black_box(vault);
            })
        });
    }
}

#[cfg(feature = "petgraph")]
fn graph_build_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_in_memory: VaultInMemory<NoteProperties> = create_vault(path);
    let vault_on_disk: VaultOnDisk<NoteProperties> = create_vault(path);
    let vault_once_cell: VaultOnceCell<NoteProperties> = create_vault(path);
    let vault_once_lock: VaultOnceLock<NoteProperties> = create_vault(path);

    c.bench_function("graph_build (VaultInMemory)", |b| {
        b.iter(|| {
            let graph = vault_in_memory.get_digraph().unwrap();
            black_box(graph);
        })
    });

    c.bench_function("graph_build (VaultOnDisk)", |b| {
        b.iter(|| {
            let graph = vault_on_disk.get_digraph().unwrap();
            black_box(graph);
        })
    });

    c.bench_function("graph_build (VaultOnceCell)", |b| {
        b.iter(|| {
            let graph = vault_once_cell.get_digraph().unwrap();
            black_box(graph);
        })
    });

    c.bench_function("graph_build (VaultOnceLock)", |b| {
        b.iter(|| {
            let graph = vault_once_lock.get_digraph().unwrap();
            black_box(graph);
        })
    });

    #[cfg(feature = "rayon")]
    {
        c.bench_function("Parallel graph_build (VaultInMemory)", |b| {
            b.iter(|| {
                let graph = vault_in_memory.par_get_digraph().unwrap();
                black_box(graph);
            })
        });

        c.bench_function("Parallel graph_build (VaultOnDisk)", |b| {
            b.iter(|| {
                let graph = vault_on_disk.par_get_digraph().unwrap();
                black_box(graph);
            })
        });

        c.bench_function("Parallel graph_build (VaultOnceLock)", |b| {
            b.iter(|| {
                let graph = vault_once_lock.par_get_digraph().unwrap();
                black_box(graph);
            })
        });
    }
}

fn get_duplicates_by_name_benchmark(c: &mut Criterion) {
    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_in_memory: VaultInMemory<NoteProperties> = create_vault(path);
    let vault_on_disk: VaultOnDisk<NoteProperties> = create_vault(path);
    let vault_once_cell: VaultOnceCell<NoteProperties> = create_vault(path);
    let vault_once_lock: VaultOnceLock<NoteProperties> = create_vault(path);

    c.bench_function("get_duplicates_notes_by_name (VaultInMemory)", |b| {
        b.iter(|| {
            let duplicated = vault_in_memory.get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });

    c.bench_function("get_duplicates_notes_by_name (VaultOnDisk)", |b| {
        b.iter(|| {
            let duplicated = vault_on_disk.get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });

    c.bench_function("get_duplicates_notes_by_name (VaultOnceCell)", |b| {
        b.iter(|| {
            let duplicated = vault_once_cell.get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });

    c.bench_function("get_duplicates_notes_by_name (VaultOnceLock)", |b| {
        b.iter(|| {
            let duplicated = vault_once_lock.get_duplicates_notes_by_name();
            black_box(duplicated);
        })
    });
}

#[cfg(feature = "digest")]
fn get_duplicates_by_content_benchmark(c: &mut Criterion) {
    use sha2::Sha256;

    let num_files = 1000;
    let links_per_file = 10;

    let temp_dir = generate_test_vault(num_files, links_per_file);
    let path = temp_dir.path();

    let vault_in_memory: VaultInMemory<NoteProperties> = create_vault(path);
    let vault_on_disk: VaultOnDisk<NoteProperties> = create_vault(path);
    let vault_once_cell: VaultOnceCell<NoteProperties> = create_vault(path);
    let vault_once_lock: VaultOnceLock<NoteProperties> = create_vault(path);

    c.bench_function(
        "get_duplicates_notes_by_content with Sha256 (VaultInMemory)",
        |b| {
            b.iter(|| {
                let duplicated = vault_in_memory
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
                let duplicated = vault_on_disk
                    .get_duplicates_notes_by_content::<Sha256>()
                    .unwrap();

                black_box(duplicated);
            })
        },
    );

    c.bench_function(
        "get_duplicates_notes_by_content with Sha256 (VaultOnceCell)",
        |b| {
            b.iter(|| {
                let duplicated = vault_once_cell
                    .get_duplicates_notes_by_content::<Sha256>()
                    .unwrap();

                black_box(duplicated);
            })
        },
    );

    c.bench_function(
        "get_duplicates_notes_by_content with Sha256 (VaultOnceLock)",
        |b| {
            b.iter(|| {
                let duplicated = vault_once_lock
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
