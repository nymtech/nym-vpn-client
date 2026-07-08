// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
use crate::diagnostic::registration::{
    lp_client::LpClientRegistration, mixnet_client::MixnetClientRegistration,
};
use nym_bandwidth_controller::{BandwidthController, BandwidthTicketProvider};
use nym_sdk::mixnet::StoragePaths;
use nym_vpn_lib_types::{
    DiagnosticRegisterParams, DiagnosticResult, RegistrationMode, RegistrationReport,
};
use nym_vpn_network_config::Network;

use std::path::PathBuf;

mod lp_client;
mod mixnet_client;
mod wireguard;

pub struct RegistrationDiagnostic;

impl RegistrationDiagnostic {
    pub async fn run_diagnostic(
        network: &Network,
        parameters: &DiagnosticRegisterParams,
    ) -> RegistrationReport {
        tracing::info!("RegistrationDiagnostic on gateway {}", parameters.gateway);

        let mut registration_report = RegistrationReport {
            mixnet_client_build: DiagnosticResult::from_value(()),
            mixnet_client_start: None,
            mixnet_ipr_connect: None,
            mixnet_based_dvpn_registration: None,
            lp_handshake: None,
            lp_based_dvpn_registration: None,
            wireguard_pings: None,
        };

        let registration_result = match parameters.registration_mode {
            RegistrationMode::Mixnet => {
                MixnetClientRegistration::register_with_mixnet_client(
                    &mut registration_report,
                    network,
                    parameters,
                )
                .await
            }
            RegistrationMode::Lp => {
                LpClientRegistration::register_with_lp(
                    &mut registration_report,
                    network,
                    parameters,
                )
                .await
            }
        };
        let Some((wireguard_configuration, keypair)) = registration_result else {
            return registration_report;
        };

        if !parameters.skip_wireguard {
            tracing::info!("Pinging over wireguard...");
            registration_report.wireguard_pings = Some(DiagnosticResult::from(
                wireguard::WireguardDiagnostic::run_diagnostic(wireguard_configuration, keypair)
                    .await,
            ));
        }

        tracing::info!("Registration diagnostic complete");
        registration_report
    }
}

async fn setup_bandwidth_provider(
    storage_path: &PathBuf,
) -> anyhow::Result<Box<dyn BandwidthTicketProvider>> {
    let storage_paths = StoragePaths::new_from_dir(storage_path)
        .map_err(|e| anyhow::anyhow!("Storage setup error : {e}"))?;
    if !storage_paths.credential_database_path.exists() {
        return Err(anyhow::anyhow!(
            "Credential database doesn't exist. Have you tried running with sudo?",
        ));
    }
    let credential_storage = storage_paths
        .persistent_credential_storage()
        .await
        .map_err(|e| anyhow::anyhow!("Credential database : {e}"))?;

    // No need for a public data fetcher. Vpn credentials are imported with the global data
    Ok(Box::new(BandwidthController::new(credential_storage)))
}
