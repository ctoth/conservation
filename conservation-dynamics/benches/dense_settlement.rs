//! Reproducible workload definitions for compiled dense settlement.
//!
//! `DenseState::settle` intentionally returns an independently owned report and
//! therefore allocates one requested `Vec` and one applied `Vec` per call.
//! `settle_discard` benchmarks isolate the reusable-workspace path when callers
//! do not need that report.
//!
//! Criterion's automatic change percentages use machine-local history under
//! `target/criterion`; old percentages are not a retained or cross-machine
//! baseline. A named local baseline can be recorded with
//! `-- --save-baseline NAME` and compared later with `-- --baseline NAME`.

use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

use conservation_core::KindId;
use conservation_dynamics::{
    DenseState, FlowSpec, FlowTopology, ProcessId, StockDefinition, StockId,
};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

struct Fixture {
    topology: Arc<FlowTopology>,
    initial: Vec<f64>,
    requested: Vec<f64>,
}

fn fixture(stock_count: usize) -> Fixture {
    circulation_fixture(stock_count, 1_000.0, &[0.25; 4])
}

fn circulation_fixture(stock_count: usize, initial: f64, branch_requests: &[f64]) -> Fixture {
    let material = KindId::new("material").unwrap();
    let stocks: Vec<_> = (0..stock_count)
        .map(|index| StockDefinition {
            id: StockId::new(format!("stock-{index}")).unwrap(),
            kind: material.clone(),
        })
        .collect();
    let fanout = branch_requests.len().min(stock_count - 1);
    let flows: Vec<_> = (0..stock_count)
        .flat_map(|source| {
            let material = material.clone();
            (1..=fanout).map(move |offset| FlowSpec {
                process: ProcessId::new(format!("move-{source}-{offset}")).unwrap(),
                kind: material.clone(),
                source: Some(StockId::new(format!("stock-{source}")).unwrap()),
                target: Some(
                    StockId::new(format!("stock-{}", (source + offset) % stock_count)).unwrap(),
                ),
            })
        })
        .collect();
    let requested: Vec<_> = (0..stock_count)
        .flat_map(|_| branch_requests.iter().copied().take(fanout))
        .collect();
    Fixture {
        topology: Arc::new(FlowTopology::new(stocks, flows).unwrap()),
        initial: vec![initial; stock_count],
        requested,
    }
}

fn boundary_fixture(stock_count: usize) -> Fixture {
    let material = KindId::new("material").unwrap();
    let stocks: Vec<_> = (0..stock_count)
        .map(|index| StockDefinition {
            id: StockId::new(format!("stock-{index}")).unwrap(),
            kind: material.clone(),
        })
        .collect();
    let flows: Vec<_> = (0..stock_count)
        .flat_map(|index| {
            let stock = StockId::new(format!("stock-{index}")).unwrap();
            [
                FlowSpec {
                    process: ProcessId::new(format!("input-{index}")).unwrap(),
                    kind: material.clone(),
                    source: None,
                    target: Some(stock.clone()),
                },
                FlowSpec {
                    process: ProcessId::new(format!("output-{index}")).unwrap(),
                    kind: material.clone(),
                    source: Some(stock),
                    target: None,
                },
            ]
        })
        .collect();
    Fixture {
        topology: Arc::new(FlowTopology::new(stocks, flows).unwrap()),
        initial: vec![1_000.0; stock_count],
        requested: vec![0.25; stock_count * 2],
    }
}

fn single_settlement(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("dense_single_settlement");
    for stock_count in [4, 64, 1_024] {
        let fixture = fixture(stock_count);
        let flow_count = fixture.requested.len();
        let mut state = DenseState::new(fixture.topology, fixture.initial).unwrap();
        group.throughput(Throughput::Elements(flow_count as u64));
        group.bench_with_input(
            BenchmarkId::new("stocks_flows", format!("{stock_count}_{flow_count}")),
            &fixture.requested,
            |bencher, requested| {
                bencher.iter(|| {
                    let report = state.settle(black_box(requested)).unwrap();
                    black_box(report.applied()[0]);
                });
            },
        );
    }
    group.finish();
}

fn long_trajectory(criterion: &mut Criterion) {
    const STEPS: usize = 10_000;
    let fixture = fixture(4);
    let mut reported_state =
        DenseState::new(fixture.topology.clone(), fixture.initial.clone()).unwrap();
    let mut discarded_state = DenseState::new(fixture.topology, fixture.initial).unwrap();
    let mut group = criterion.benchmark_group("dense_long_trajectory");
    group.throughput(Throughput::Elements(
        (fixture.requested.len() * STEPS) as u64,
    ));
    group.bench_function("4_stocks_12_flows_10000_steps", |bencher| {
        bencher.iter(|| {
            for _ in 0..STEPS {
                let report = reported_state
                    .settle(black_box(&fixture.requested))
                    .unwrap();
                black_box(report.applied()[0]);
            }
        });
    });
    group.bench_function("4_stocks_12_flows_10000_steps_discard", |bencher| {
        bencher.iter(|| {
            for _ in 0..STEPS {
                discarded_state
                    .settle_discard(black_box(&fixture.requested))
                    .unwrap();
                black_box(discarded_state.amounts()[0]);
            }
        });
    });
    group.finish();
}

fn repeated_batch(criterion: &mut Criterion) {
    const STATE_COUNT: usize = 64;
    const STEPS: usize = 100;
    let fixture = fixture(64);
    let mut reported_states: Vec<_> = (0..STATE_COUNT)
        .map(|_| DenseState::new(fixture.topology.clone(), fixture.initial.clone()).unwrap())
        .collect();
    let mut discarded_states: Vec<_> = (0..STATE_COUNT)
        .map(|_| DenseState::new(fixture.topology.clone(), fixture.initial.clone()).unwrap())
        .collect();
    let mut group = criterion.benchmark_group("dense_repeated_batch");
    group.throughput(Throughput::Elements(
        (STATE_COUNT * STEPS * fixture.requested.len()) as u64,
    ));
    group.bench_function("64_states_64_stocks_256_flows_100_steps", |bencher| {
        bencher.iter(|| {
            for _ in 0..STEPS {
                for state in &mut reported_states {
                    let report = state.settle(black_box(&fixture.requested)).unwrap();
                    black_box(report.applied()[0]);
                }
            }
        });
    });
    group.bench_function(
        "64_states_64_stocks_256_flows_100_steps_discard",
        |bencher| {
            bencher.iter(|| {
                for _ in 0..STEPS {
                    for state in &mut discarded_states {
                        state.settle_discard(black_box(&fixture.requested)).unwrap();
                        black_box(state.amounts()[0]);
                    }
                }
            });
        },
    );
    group.finish();
}

fn representative_workloads(criterion: &mut Criterion) {
    let workloads = [
        (
            "proportional_limiting_64_stocks_256_flows",
            circulation_fixture(64, 1.0, &[1.0; 4]),
        ),
        (
            "boundary_accounts_64_stocks_128_flows",
            boundary_fixture(64),
        ),
        (
            "dynamic_range_64_stocks_256_flows",
            circulation_fixture(64, 1.0e12, &[1.0e-12, 1.0e-3, 1.0e3, 1.0e12]),
        ),
    ];
    let mut group = criterion.benchmark_group("dense_representative_discard");
    for (name, fixture) in workloads {
        let mut state = DenseState::new(fixture.topology, fixture.initial).unwrap();
        group.throughput(Throughput::Elements(fixture.requested.len() as u64));
        group.bench_with_input(name, &fixture.requested, |bencher, requested| {
            bencher.iter(|| {
                state.settle_discard(black_box(requested)).unwrap();
                black_box(state.amounts()[0]);
            });
        });
    }
    group.finish();
}

fn benchmark_config() -> Criterion {
    Criterion::default()
        .sample_size(10)
        .measurement_time(Duration::from_secs(3))
}

criterion_group! {
    name = benches;
    config = benchmark_config();
    targets = single_settlement, long_trajectory, repeated_batch, representative_workloads
}
criterion_main!(benches);
