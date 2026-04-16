// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
use anyhow::{Context, Result};

static EMBEDDED_COUNTRY_DOMAINS: &[(&str, &[u8])] =
    &[("CN", include_bytes!("../../builtin/CN-domain.txt.gz"))];

pub struct DomainSet {
    reversed: Vec<String>,
}

impl DomainSet {
    pub async fn load(excluded_countries: &[String]) -> Result<Self> {
        let mut all_reversed: Vec<String> = Vec::new();

        for code in excluded_countries {
            let upper = code.to_uppercase();
            let Some(&(_, gz)) = EMBEDDED_COUNTRY_DOMAINS
                .iter()
                .find(|&&(cc, _)| cc == upper.as_str())
            else {
                tracing::debug!("No embedded domain list for country {upper}; skipping");
                continue;
            };

            let text = super::decompress_gz(gz)
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
        // Binary search for the insertion point of `rev`.
        let idx = self
            .reversed
            .partition_point(|entry| entry.as_str() <= rev.as_str());

        // Check the entry just before `idx` (exact match or rev starts with entry+".")
        if idx > 0 {
            let prev = &self.reversed[idx - 1];
            if is_suffix_match(&rev, prev) {
                return true;
            }
        }
        // Also check the entry at `idx` itself (rev might equal it exactly).
        if idx < self.reversed.len() {
            let at = &self.reversed[idx];
            if is_suffix_match(&rev, at) {
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

fn reverse_labels(domain: &str) -> String {
    let labels: Vec<&str> = domain.trim_end_matches('.').split('.').collect();
    labels.into_iter().rev().collect::<Vec<_>>().join(".")
}

#[inline]
fn is_suffix_match(rev_host: &str, rev_entry: &str) -> bool {
    rev_host == rev_entry
        || (rev_host.len() > rev_entry.len()
            && rev_host.as_bytes().get(rev_entry.len()) == Some(&b'.')
            && rev_host.starts_with(rev_entry))
}
