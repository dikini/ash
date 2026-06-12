//! Benchmark daemon mode latency improvements

use ash_mcp::daemon::DaemonState;
use ash_parser::parse_surface_file;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::NamedTempFile;

fn write_test_ash_file(content: &str) -> NamedTempFile {
    use std::io::Write;
    let mut file = NamedTempFile::new().unwrap();
    writeln!(file, "{}", content).unwrap();
    file
}

fn benchmark_daemon_cached_parse(c: &mut Criterion) {
    let test_content = r#"
import act;

def foo(x: Int) -> Int {
    return x + 1
}

def bar(y: Int) -> Int {
    return y * 2
}
"#;

    let temp_file = write_test_ash_file(test_content);
    let path = PathBuf::from(temp_file.path());

    let daemon = DaemonState::new();

    // First request (cache miss)
    daemon
        .parse_file_cached(&path)
        .expect("First parse should succeed");

    let mut group = c.benchmark_group("daemon_cache");
    group.measurement_time(Duration::from_secs(5));

    // Benchmark cached requests
    group.bench_function("cache_hit", |b| {
        b.iter(|| {
            black_box(daemon.parse_file_cached(black_box(&path)).unwrap());
        })
    });

    group.finish();
}

fn benchmark_daemon_first_parse(c: &mut Criterion) {
    let test_content = r#"
import act;

def foo(x: Int) -> Int {
    return x + 1
}

def bar(y: Int) -> Int {
    return y * 2
}
"#;

    let mut group = c.benchmark_group("daemon_first_parse");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("cache_miss", |b| {
        b.iter(|| {
            let temp_file = write_test_ash_file(test_content);
            let path = PathBuf::from(temp_file.path());
            let daemon = DaemonState::new();
            black_box(daemon.parse_file_cached(black_box(&path)).unwrap());
        })
    });

    group.finish();
}

fn benchmark_baseline_parse(c: &mut Criterion) {
    let test_content = r#"
import act;

def foo(x: Int) -> Int {
    return x + 1
}

def bar(y: Int) -> Int {
    return y * 2
}
"#;

    let mut group = c.benchmark_group("baseline_parse");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("direct_parse", |b| {
        b.iter(|| {
            let temp_file = write_test_ash_file(test_content);
            let _path = PathBuf::from(temp_file.path());
            let content = std::fs::read_to_string(temp_file.path()).unwrap();
            black_box(parse_surface_file(&content).unwrap());
        })
    });

    group.finish();
}

fn benchmark_cache_size_scaling(c: &mut Criterion) {
    let test_content = r#"
import act;

def foo(x: Int) -> Int {
    return x + 1
}
"#;

    let mut group = c.benchmark_group("cache_scaling");
    group.measurement_time(Duration::from_secs(5));

    for cache_size in [10, 25, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(cache_size),
            cache_size,
            |b, &size| {
                b.iter(|| {
                    let daemon = DaemonState::new();
                    for i in 0..size {
                        let temp_file =
                            write_test_ash_file(&format!("// File {}\n{}", i, test_content));
                        let path = PathBuf::from(temp_file.path());
                        daemon.parse_file_cached(&path).unwrap();
                    }
                    // Access the last file (should still be in cache)
                    let temp_file =
                        write_test_ash_file(&format!("// Final file\n{}", test_content));
                    let path = PathBuf::from(temp_file.path());
                    black_box(daemon.parse_file_cached(black_box(&path)).unwrap());
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_daemon_cached_parse,
    benchmark_daemon_first_parse,
    benchmark_baseline_parse,
    benchmark_cache_size_scaling
);
criterion_main!(benches);
