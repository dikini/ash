//! Benchmark cache performance (hit rates, eviction, mtime invalidation)

use ash_mcp::daemon::DaemonState;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

fn write_test_ash_file(id: usize) -> NamedTempFile {
    use std::io::Write;
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "// File {}", id).unwrap();
    writeln!(file, "def foo_{}() -> Int {{ return {} }}", id, id).unwrap();
    file
}

fn benchmark_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_rate");
    group.measurement_time(Duration::from_secs(5));

    // Sequential access (high hit rate)
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let daemon = DaemonState::new();
            for i in 0..10 {
                let temp_file = write_test_ash_file(i);
                let path = PathBuf::from(temp_file.path());
                // Parse once
                daemon.parse_file_cached(&path).unwrap();
                // Parse again (should be cache hit)
                daemon.parse_file_cached(&path).unwrap();
            }
        })
    });

    // Random access (lower hit rate)
    group.bench_function("random", |b| {
        b.iter(|| {
            let daemon = DaemonState::new();
            let indices: Vec<usize> = (0..50).collect();
            for _ in 0..100 {
                let idx = indices[rand::random::<usize>() % indices.len()];
                let temp_file = write_test_ash_file(idx);
                let path = PathBuf::from(temp_file.path());
                daemon.parse_file_cached(&path).unwrap();
            }
        })
    });

    group.finish();
}

fn benchmark_cache_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_eviction");
    group.measurement_time(Duration::from_secs(5));

    // Fill cache beyond capacity (50 entries)
    group.bench_function("overfill", |b| {
        b.iter(|| {
            let daemon = DaemonState::new();
            for i in 0..100 {
                let temp_file = write_test_ash_file(i);
                let path = PathBuf::from(temp_file.path());
                daemon.parse_file_cached(&path).unwrap();
            }
            // Access early file (should have been evicted)
            let temp_file = write_test_ash_file(0);
            let path = PathBuf::from(temp_file.path());
            black_box(daemon.parse_file_cached(black_box(&path)).unwrap());
        })
    });

    group.finish();
}

fn benchmark_mtime_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("mtime_invalidation");
    group.measurement_time(Duration::from_secs(3));

    group.bench_function("unchanged", |b| {
        b.iter(|| {
            let daemon = DaemonState::new();
            let temp_file = write_test_ash_file(1);
            let path = PathBuf::from(temp_file.path());

            // First parse (cache miss)
            daemon.parse_file_cached(&path).unwrap();
            // Second parse with same mtime (cache hit)
            black_box(daemon.parse_file_cached(black_box(&path)).unwrap());
        })
    });

    group.finish();
}

fn benchmark_concurrent_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_access");
    group.measurement_time(Duration::from_secs(5));

    for threads in [1, 2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(threads),
            threads,
            |b, &num_threads| {
                b.iter(|| {
                    let daemon = std::sync::Arc::new(DaemonState::new());
                    let mut handles = vec![];

                    for _ in 0..num_threads {
                        let daemon_clone = daemon.clone();
                        let handle = thread::spawn(move || {
                            for i in 0..20 {
                                let temp_file = write_test_ash_file(i);
                                let path = PathBuf::from(temp_file.path());
                                let _ = daemon_clone.parse_file_cached(&path);
                            }
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        handle.join().unwrap();
                    }
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_cache_hit_rate,
    benchmark_cache_eviction,
    benchmark_mtime_invalidation,
    benchmark_concurrent_access
);
criterion_main!(benches);
