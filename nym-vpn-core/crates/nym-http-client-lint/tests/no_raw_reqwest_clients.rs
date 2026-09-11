// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Guards against a bug class that hid for a while on Android: any code that builds a
//! `reqwest::Client` without going through `nym_http_api_client`'s registry silently skips
//! every cross-cutting config registered there — including the Android-only override that
//! swaps in a webpki-roots TLS backend so `rustls-platform-verifier` (which needs an explicit,
//! easy-to-forget JNI init call) is never consulted. Two such bypasses shipped in
//! `nym-file-updater` and `nym-wg-metadata-client` and caused
//! "Expect rustls-platform-verifier to be initialized" panics in production. This test fails
//! the build if a new one is introduced.
//!
//! It intentionally only does a line-based text scan, not real parsing: it's a cheap tripwire,
//! not a linter. Test-only code (files under a `tests/` directory, or anything after a file's
//! first `#[cfg(test)]` module) is exempt, since it never runs on a real device.

use std::fs;
use std::path::Path;

use regex::Regex;

#[test]
fn production_code_never_bypasses_the_http_client_registry() {
    // Canonicalized so joined child paths don't retain a literal `nym-http-client-lint/..`
    // component, which would otherwise match the exclusion check below for every file.
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();

    let banned = Regex::new(concat!(
        r"reqwest\s*::\s*Client\s*::\s*(builder|new)\s*\(",
        r"|reqwest\s*::\s*ClientBuilder\s*::\s*new\s*\(",
        r"|ReqwestClientBuilder\s*::\s*new\s*\(",
    ))
    .unwrap();
    let cfg_test_mod = Regex::new(r"(?m)^\s*#\[cfg\(test\)\]").unwrap();
    // `use reqwest::Client;` (optionally renamed via `as`, or grouped in braces) lets code
    // call the constructor unqualified afterwards, which the fully-qualified regex above
    // would miss. Resolve just that one level of indirection rather than the general case.
    let use_reqwest =
        Regex::new(r"(?m)^\s*use\s+reqwest\s*::\s*(?:\{([^}]*)\}|(\w+(?:\s*as\s*\w+)?))\s*;")
            .unwrap();
    let import_item = Regex::new(r"(ClientBuilder|Client)(?:\s*as\s*(\w+))?").unwrap();

    let mut violations = Vec::new();
    visit_rust_files(&crates_dir, &mut |path, contents| {
        // This lint's own doc comments above reference the banned patterns as prose.
        if path
            .components()
            .any(|c| c.as_os_str() == "nym-http-client-lint")
        {
            return;
        }
        // Files under a `tests/` directory never run on a real device.
        if path.components().any(|c| c.as_os_str() == "tests") {
            return;
        }

        // By convention test modules live at the end of a file, so only scan what
        // comes before the first `#[cfg(test)]`.
        let scope = match cfg_test_mod.find(contents) {
            Some(m) => &contents[..m.start()],
            None => contents,
        };

        let mut client_aliases = Vec::new();
        let mut builder_aliases = Vec::new();
        for cap in use_reqwest.captures_iter(scope) {
            let body = cap
                .get(1)
                .or_else(|| cap.get(2))
                .map(|m| m.as_str())
                .unwrap_or_default();
            for item in body.split(',') {
                let Some(m) = import_item.captures(item) else {
                    continue;
                };
                let alias = m
                    .get(2)
                    .map(|a| a.as_str().to_string())
                    .unwrap_or_else(|| m[1].to_string());
                if &m[1] == "ClientBuilder" {
                    builder_aliases.push(alias);
                } else {
                    client_aliases.push(alias);
                }
            }
        }
        let alias_banned = (!client_aliases.is_empty() || !builder_aliases.is_empty()).then(|| {
            let mut parts: Vec<String> = client_aliases
                .iter()
                .map(|a| format!(r"\b{}\s*::\s*(builder|new)\s*\(", regex::escape(a)))
                .collect();
            parts.extend(
                builder_aliases
                    .iter()
                    .map(|a| format!(r"\b{}\s*::\s*new\s*\(", regex::escape(a))),
            );
            Regex::new(&parts.join("|")).unwrap()
        });

        for (line_no, line) in scope.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            let aliased_hit = alias_banned.as_ref().is_some_and(|re| re.is_match(line));
            if banned.is_match(line) || aliased_hit {
                violations.push(format!("{}:{}: {}", path.display(), line_no + 1, trimmed));
            }
        }
    });

    assert!(
        violations.is_empty(),
        "found raw reqwest client construction outside nym_http_api_client's registry.\n\
         Use `nym_http_api_client::registry::default_builder()` (or `build_client()`) instead \
         of `reqwest::Client::builder()` / `reqwest::Client::new()` / `ReqwestClientBuilder::new()`, \
         so platform-specific overrides (e.g. Android's TLS backend) still apply:\n{}",
        violations.join("\n")
    );
}

fn visit_rust_files(dir: &Path, f: &mut dyn FnMut(&Path, &str)) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit_rust_files(&path, f);
        } else if path.extension().is_some_and(|ext| ext == "rs")
            && let Ok(contents) = fs::read_to_string(&path)
        {
            f(&path, &contents);
        }
    }
}
