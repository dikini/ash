use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_parse_simple_workflow(c: &mut Criterion) {
    let input = r#"
workflow test {
    observe capability "sensor" as data
    orient {
        analyze data with classifier
    }
    decide {
        if data.value > 10 then action "alert"
    }
}
"#;
    
    c.bench_function("parse_simple_workflow", |b| {
        b.iter(|| {
            // Placeholder - actual parsing will be implemented
            black_box(input.len())
        });
    });
}

fn bench_parse_complex_workflow(c: &mut Criterion) {
    let input = r#"
workflow complex {
    observe capability "db" as records
    
    par {
        branch a {
            orient { filter records where valid }
            decide { 
                if count > 100 then action "batch_process" 
            }
        }
        branch b {
            orient { sort records by timestamp }
            decide { 
                if needs_archival then action "archive" 
            }
        }
    }
    
    with capability "notifier" {
        act notify all_complete
    }
}
"#;
    
    c.bench_function("parse_complex_workflow", |b| {
        b.iter(|| {
            black_box(input.len())
        });
    });
}

fn bench_parse_nested_workflows(c: &mut Criterion) {
    let input = r#"
workflow outer {
    observe capability "api" as request
    orient {
        match request.type {
            Type::A => workflow inner_a {
                orient { validate request }
                decide { action "process_a" }
            }
            Type::B => workflow inner_b {
                orient { transform request }
                decide { action "process_b" }
            }
        }
    }
}
"#;
    
    c.bench_function("parse_nested_workflows", |b| {
        b.iter(|| {
            black_box(input.len())
        });
    });
}

criterion_group!(
    parser_benches,
    bench_parse_simple_workflow,
    bench_parse_complex_workflow,
    bench_parse_nested_workflows
);
criterion_main!(parser_benches);
