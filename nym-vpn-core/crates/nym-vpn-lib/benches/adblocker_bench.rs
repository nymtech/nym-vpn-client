use adblock::lists::{ParseOptions, ParsedFilter, RuleTypes, parse_filter};
use criterion::{Criterion, criterion_group, criterion_main};
use futures::{StreamExt, pin_mut};
use rand::Rng;
use rand::seq::SliceRandom;
use std::path::Path;
use tokio::runtime::Runtime;

use nym_vpn_lib::adblocker::{
    engines::{AdBlockEngine, BraveAdblockEngine, SimpleAdBlockEngine},
    file_manager::{SOURCES, Source, init_tests},
};
use nym_vpn_lib::dns_filter::DnsFilterT;

fn bench_brave_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tempdir = rt.block_on(init_tests()).unwrap();
    let engine = BraveAdblockEngine::default();
    rt.block_on(engine.load_filters(tempdir.path())).unwrap();
    let domains = rt.block_on(prepare_domains(tempdir.path()));

    let mut next_domain = 0usize;
    c.bench_function("Brave Engine - Block", |b| {
        b.to_async(&rt).iter(|| {
            let domain = &domains[next_domain % domains.len()];
            next_domain = next_domain.wrapping_add(1);
            async {
                let decision = engine.should_block(std::hint::black_box(domain)).await;
                std::hint::black_box(decision)
            }
        });
    });
}

fn bench_simple_engine(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tempdir = rt.block_on(init_tests()).unwrap();
    let engine = SimpleAdBlockEngine::new(tempdir.path().join("adblocker.db"));
    rt.block_on(engine.load_filters(tempdir.path())).unwrap();
    let domains = rt.block_on(prepare_domains(tempdir.path()));

    let mut next_domain = 0usize;
    c.bench_function("Simple Engine - Block", |b| {
        b.to_async(&rt).iter(|| {
            let domain = &domains[next_domain % domains.len()];
            next_domain = next_domain.wrapping_add(1);
            async {
                let decision = engine.should_block(std::hint::black_box(domain)).await;
                std::hint::black_box(decision)
            }
        });
    });
}

fn bench_brave_engine_miss(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    const NUM_MISS_DOMAINS: usize = 35000;
    let tempdir = rt.block_on(init_tests()).unwrap();
    let engine = BraveAdblockEngine::default();
    rt.block_on(engine.load_filters(tempdir.path())).unwrap();

    let miss_domains = prepare_random_miss_domains(NUM_MISS_DOMAINS);
    let mut next_domain = 0usize;
    c.bench_function("Brave Engine - Miss", |b| {
        b.to_async(&rt).iter(|| {
            let domain = &miss_domains[next_domain % miss_domains.len()];
            next_domain = next_domain.wrapping_add(1);
            async {
                let decision = engine.should_block(std::hint::black_box(domain)).await;
                std::hint::black_box(decision)
            }
        });
    });
}

fn bench_simple_engine_miss(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    const NUM_MISS_DOMAINS: usize = 35000;
    let tempdir = rt.block_on(init_tests()).unwrap();
    let engine = SimpleAdBlockEngine::new(tempdir.path().join("adblocker.db"));
    rt.block_on(engine.load_filters(tempdir.path())).unwrap();

    let miss_domains = prepare_random_miss_domains(NUM_MISS_DOMAINS);
    let mut next_domain = 0usize;
    c.bench_function("Simple Engine - Miss", |b| {
        b.to_async(&rt).iter(|| {
            let domain = &miss_domains[next_domain % miss_domains.len()];
            next_domain = next_domain.wrapping_add(1);
            async {
                let decision = engine.should_block(std::hint::black_box(domain)).await;
                std::hint::black_box(decision)
            }
        });
    });
}

fn prepare_random_miss_domains(count: usize) -> Vec<String> {
    const LABEL_LEN: usize = 12;
    let mut rng = rand::thread_rng();

    let mut domains = Vec::with_capacity(count);
    for idx in 0..count {
        let label: String = (0..LABEL_LEN)
            .map(|_| (b'a' + rng.gen_range(0..26)) as char)
            .collect();
        domains.push(format!("{label}-{idx}.adblock-bench.invalid"));
    }

    domains
}

async fn prepare_domains(cache_dir: &Path) -> Vec<String> {
    const NUM_DOMAINS: usize = 35000;
    let mut domains = vec![];

    for source in SOURCES {
        let data_path = cache_dir.join(source.file_name);
        let line_stream = Source::stream_lines(&data_path).take(NUM_DOMAINS);
        let opts = ParseOptions {
            format: source.filterset_format,
            rule_types: RuleTypes::NetworkOnly,
            ..Default::default()
        };
        pin_mut!(line_stream);
        while let Some(Ok(line)) = line_stream.next().await {
            if let Ok(ParsedFilter::Network(filter)) = parse_filter(&line, false, opts)
                && let Some(ref domain) = filter.hostname
            {
                // Convert to lowercase for case-insensitive comparison
                domains.push(domain.to_lowercase());
            }
        }
    }

    let mut rng = rand::thread_rng();

    let mut random_domains = Vec::with_capacity(domains.len());
    for _ in 0..NUM_DOMAINS * 2 {
        let random_domain = domains.choose(&mut rng).unwrap();
        random_domains.push(format!("bob.{}", random_domain));
    }

    domains.extend(random_domains);
    domains.shuffle(&mut rng);

    domains
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10000);
    targets =
    bench_brave_engine,
    bench_simple_engine,
    bench_brave_engine_miss,
    bench_simple_engine_miss
);
criterion_main!(benches);
