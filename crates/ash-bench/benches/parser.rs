use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_simple_program(c: &mut Criterion) {
    let input = r#"
fn main() -> Int {
    let value = 12;
    if value > 10 { 1 } else { 0 }
}
"#;

    c.bench_function("parse_simple_program", |b| {
        b.iter(|| {
            // Placeholder - actual parsing will be implemented
            black_box(input.len())
        });
    });
}

fn bench_parse_complex_program(c: &mut Criterion) {
    let input = r#"
fn normalize(count: Int) -> Int {
    if count > 100 { count } else { 0 }
}

fn main() -> Int {
    let records = 128;
    normalize(records)
}
"#;

    c.bench_function("parse_complex_program", |b| {
        b.iter(|| {
            black_box(input.len())
        });
    });
}

fn bench_parse_nested_functions(c: &mut Criterion) {
    let input = r#"
type Request = A | B;

fn process(request: Request) -> Int {
    match request {
        A => 1,
        B => 2,
    }
}

fn main() -> Int {
    process(A)
}
"#;

    c.bench_function("parse_nested_functions", |b| {
        b.iter(|| {
            black_box(input.len())
        });
    });
}

criterion_group!(
    parser_benches,
    bench_parse_simple_program,
    bench_parse_complex_program,
    bench_parse_nested_functions
);
criterion_main!(parser_benches);
