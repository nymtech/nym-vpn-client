// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
use std::path::Path;

use anyhow::{Context, Result};

pub struct DomainSet {
    reversed: Vec<String>,
}

impl DomainSet {
    pub async fn load(excluded_countries: &[String], data_dir: &Path) -> Result<Self> {
        let mut all_reversed: Vec<String> = Vec::new();

        for code in excluded_countries {
            let upper = code.to_uppercase();
            let gz = load_country_domain_gz(&upper, data_dir).await?;

            let text = super::decompress_gz(&gz)
                .await
                .with_context(|| format!("Failed to decompress domain list for {upper}"))?;

            let entries = text
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(reverse_labels);

            all_reversed.extend(entries);
            tracing::debug!("Loaded domain exclusion list for {upper}");
        }

        all_reversed.sort_unstable();
        all_reversed.dedup();
        tracing::info!(count = all_reversed.len(), "Loaded domain exclusion list");
        Ok(Self {
            reversed: all_reversed,
        })
    }

    /// Returns `true` if `host` is an exact match or a subdomain of any entry in the list.
    pub fn is_excluded(&self, host: &str) -> bool {
        // Strip trailing dot (FQDN form).
        let host = host.trim_end_matches('.');
        if host.is_empty() {
            return false;
        }
        let rev = reverse_labels(host);
        // Check each ancestor domain (and the host itself) for an exact match, from the
        // top-level label down. A sibling subdomain can sort between an ancestor entry and
        // `rev`, so only checking the binary search insertion point's neighbors is not enough.
        let mut prefix = String::new();
        for label in rev.split('.') {
            if !prefix.is_empty() {
                prefix.push('.');
            }
            prefix.push_str(label);
            if self.reversed.binary_search(&prefix).is_ok() {
                return true;
            }
        }
        false
    }

    #[cfg(test)]
    pub(super) fn from_text(text: &str) -> Result<Self> {
        let mut reversed: Vec<String> = text
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .map(reverse_labels)
            .collect();
        reversed.sort_unstable();
        Ok(Self { reversed })
    }
}

/// Load the gzip bytes for a country's domain list: file on disk first, embedded fallback.
async fn load_country_domain_gz(country_code: &str, data_dir: &Path) -> Result<Vec<u8>> {
    let path = data_dir.join(format!("{country_code}-domain.txt.gz"));

    match tokio::fs::read(&path).await {
        Ok(bytes) => {
            tracing::debug!("Loaded updated domain list from '{}'", path.display());
            Ok(bytes)
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => embedded_domain_gz(country_code)
            .map(|b| b.to_vec())
            .ok_or_else(|| anyhow::anyhow!("No domain list available for country {country_code}")),
        Err(err) => {
            tracing::warn!(
                "Could not read domain list from '{}': {err}; using embedded data",
                path.display(),
            );
            embedded_domain_gz(country_code)
                .map(|b| b.to_vec())
                .ok_or_else(|| {
                    anyhow::anyhow!("No embedded domain list available for country {country_code}")
                })
        }
    }
}

fn embedded_domain_gz(country_code: &str) -> Option<&'static [u8]> {
    let file_name = format!("{country_code}-domain.txt.gz");
    crate::file_manager::SOURCES
        .iter()
        .find(|s| s.file_name == file_name.as_str())
        .map(|s| s.builtin)
}

fn reverse_labels(domain: &str) -> String {
    let labels: Vec<&str> = domain.trim_end_matches('.').split('.').collect();
    labels.into_iter().rev().collect::<Vec<_>>().join(".")
}
