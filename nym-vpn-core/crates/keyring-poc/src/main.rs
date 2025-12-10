// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use clap::Parser;
use keyring::Entry;
use nym_crypto::asymmetric::ed25519::{KeyPair, PrivateKey};
use rand::rngs::OsRng;

#[tokio::main]
async fn main() -> Result<()> {
    let args = ProgramArgs::parse();

    args.command.execute().await
}

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct ProgramArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Store a freshly generated keypair
    GenerateAndStore,

    /// Retrieve the vpnd keypair
    Get,
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        match self {
            Command::GenerateAndStore => Self::generate_and_store()?,
            Command::Get => Self::get()?,
        };
        Ok(())
    }

    fn generate_and_store() -> Result<KeyPair> {
        let keypair = KeyPair::new(&mut OsRng);
        let entry = Entry::new("nym-vpn", "vpnd")?;
        entry.set_secret(&keypair.private_key().to_bytes())?;
        println!(
            "Stored secret for public key: {}",
            keypair.public_key().to_base58_string()
        );

        Ok(keypair)
    }

    fn get() -> Result<KeyPair> {
        let entry = Entry::new("nym-vpn", "vpnd")?;
        let keypair_bytes = entry.get_secret()?;
        let keypair = KeyPair::from(PrivateKey::from_bytes(&keypair_bytes)?);
        println!(
            "Retrieved secret for public key: {}",
            keypair.public_key().to_base58_string()
        );

        Ok(keypair)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_and_get() {
        let vpnd_stored = Command::generate_and_store().unwrap();
        let client_retrieved = Command::get().unwrap();
        assert_eq!(vpnd_stored.public_key(), client_retrieved.public_key());
    }
}
