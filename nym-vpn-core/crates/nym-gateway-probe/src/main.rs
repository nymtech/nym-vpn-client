// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
mod run;

#[cfg(unix)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match run::run().await {
        Ok(ref result) => {
            let json = serde_json::to_string_pretty(result)?;
            println!("{}", json);
        }
        Err(err) => {
            eprintln!("An error occurred: {err}");
            std::process::exit(1)
        }
    }
    Ok(())
}

#[cfg(not(unix))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    eprintln!("This tool is only supported on Unix systems");
    std::process::exit(1)
}
