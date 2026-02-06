// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tracing_subscriber::{EnvFilter, filter::Directive};

use crate::cli::CliArgs;

pub(crate) fn setup_tracing_logger(args: &CliArgs) -> anyhow::Result<()> {
    fn directive_checked(directive: impl Into<String>) -> anyhow::Result<Directive> {
        directive.into().parse().map_err(From::from)
    }
    if args.no_log {
        return Ok(());
    }

    let log_builder = tracing_subscriber::fmt()
        // Use a more compact, abbreviated log format
        .compact()
        // Display source code file paths
        .with_file(true)
        // Display source code line numbers
        .with_line_number(true)
        // Don't display the event's target (module path)
        .with_target(false);

    let mut filter = EnvFilter::builder()
        .with_default_directive(args.verbosity_level().into())
        .from_env_lossy();

    // these crates are more granularly filtered
    let info_crates = &["hickory_proto", "hickory_resolver", "h2", "rustls"];

    for crate_name in info_crates {
        filter = filter.add_directive(directive_checked(format!("{crate_name}=info"))?)
    }

    let log_level_hint = filter.max_level_hint();

    log_builder.with_env_filter(filter).init();
    tracing::info!("Log level: {:?}", log_level_hint);

    Ok(())
}
