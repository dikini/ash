use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ash_core::effect::Effect;

fn bench_effect_join(c: &mut Criterion) {
    let effects = [
        Effect::Epistemic,
        Effect::Deliberative,
        Effect::Evaluative,
        Effect::Operational,
    ];
    
    c.bench_function("effect_join_epistemic_deliberative", |b| {
        b.iter(|| {
            black_box(Effect::Epistemic.join(Effect::Deliberative))
        });
    });
    
    c.bench_function("effect_join_all_pairs", |b| {
        b.iter(|| {
            for e1 in &effects {
                for e2 in &effects {
                    black_box(e1.join(*e2));
                }
            }
        });
    });
}

fn bench_effect_meet(c: &mut Criterion) {
    let effects = [
        Effect::Epistemic,
        Effect::Deliberative,
        Effect::Evaluative,
        Effect::Operational,
    ];
    
    c.bench_function("effect_meet_operational_evaluative", |b| {
        b.iter(|| {
            black_box(Effect::Operational.meet(Effect::Evaluative))
        });
    });
    
    c.bench_function("effect_meet_all_pairs", |b| {
        b.iter(|| {
            for e1 in &effects {
                for e2 in &effects {
                    black_box(e1.meet(*e2));
                }
            }
        });
    });
}

fn bench_effect_comparison(c: &mut Criterion) {
    c.bench_function("effect_partial_cmp", |b| {
        b.iter(|| {
            black_box(Effect::Epistemic.partial_cmp(&Effect::Operational));
        });
    });
    
    c.bench_function("effect_le", |b| {
        b.iter(|| {
            black_box(Effect::Epistemic <= Effect::Deliberative);
        });
    });
}

criterion_group!(
    effect_lattice_benches,
    bench_effect_join,
    bench_effect_meet,
    bench_effect_comparison
);
criterion_main!(effect_lattice_benches);
