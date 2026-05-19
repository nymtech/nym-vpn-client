// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use adblock::lists::{ParseOptions, ParsedFilter, RuleTypes, parse_filter};
use futures::{StreamExt, pin_mut};
use rand::seq::SliceRandom;
use std::path::Path;

use super::{
    engines::{AdBlockEngine, BraveAdblockEngine, SimpleAdBlockEngine},
    file_manager::{SOURCES, Source, tests::init_tests},
};
use crate::dns_filter::DnsFilterT;

#[tokio::test]
async fn bench_brave_engine() {
    let tempdir = init_tests().await.unwrap();
    let engine = BraveAdblockEngine::default();
    engine.load_filters(tempdir.path()).await.unwrap();

    let domains = prepare_domains(tempdir.path()).await;
    let start = std::time::Instant::now();
    for domain in domains.iter() {
        let _ = engine.should_block(&domain);
    }
    println!(
        "Duration: {:?} on {} domains",
        start.elapsed(),
        domains.len()
    );
}

#[tokio::test]
async fn bench_simple_engine() {
    let tempdir = init_tests().await.unwrap();
    let engine = SimpleAdBlockEngine::new(tempdir.path().join("adblocker.db"));
    engine.load_filters(tempdir.path()).await.unwrap();

    let domains = prepare_domains(tempdir.path()).await;
    let start = std::time::Instant::now();
    for domain in domains.iter() {
        let _ = engine.should_block(&domain);
    }
    println!(
        "Duration: {:?} on {} domains",
        start.elapsed(),
        domains.len()
    );
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
