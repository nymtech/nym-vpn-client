// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{pin::pin, time::Duration};

use futures::{FutureExt, future::Fuse};
use rand::{distributions::Uniform, prelude::Distribution};
use sysinfo::System;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_util::sync::CancellationToken;

use nym_statistics_api_client::StatisticsApiClient;
use nym_statistics_common::{
    generate_vpn_client_stats_id,
    report::vpn_client::{ActiveDeviceReport, StaticInformationReport, VpnClientStatsReportV2},
};
use nym_vpn_lib_types::NetworkStatisticsConfig;

use crate::{
    commands::ConfigCommand, error::Error, events::ReportSendingEvent, storage::StatsStorage,
};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum TunnelState {
    Connected,
    #[default]
    Disconnected,
    Standby,
}

#[derive(Default, Clone, Copy)]
struct SendingConfig {
    tunnel_state: TunnelState,
    allow_direct_sending: bool,
}

impl SendingConfig {
    const SMALL_SENDING_DELAY_MAX_SECS: u64 = 30;
    const BIG_SENDING_DELAY_MIN_MINS: u64 = 15;
    const BIG_SENDING_DELAY_MAX_MINS: u64 = 120;
    const MAX_RETRIES: u32 = 5;

    fn allows_sending(&self) -> bool {
        // Either we are connected, or we are disconnected and direct sending is allowed
        (self.tunnel_state == TunnelState::Connected)
            || (self.allow_direct_sending && self.tunnel_state == TunnelState::Disconnected)
    }

    fn random_small_delay() -> Duration {
        let random_delay_secs =
            Uniform::new_inclusive(0, SendingConfig::SMALL_SENDING_DELAY_MAX_SECS)
                .sample(&mut rand::thread_rng());
        Duration::from_secs(random_delay_secs)
    }

    fn random_big_delay() -> Duration {
        let random_delay_mins = Uniform::new_inclusive(
            SendingConfig::BIG_SENDING_DELAY_MIN_MINS,
            SendingConfig::BIG_SENDING_DELAY_MAX_MINS,
        )
        .sample(&mut rand::thread_rng());
        Duration::from_secs(random_delay_mins * 60)
    }

    // Return Some(new_value) if `allows_sending` changed, None otherwise
    fn update(&mut self, event: ReportSendingEvent) -> Option<bool> {
        let old_value = self.allows_sending();
        match event {
            ReportSendingEvent::Connected => self.tunnel_state = TunnelState::Connected,
            ReportSendingEvent::Disconnected => self.tunnel_state = TunnelState::Disconnected,
            ReportSendingEvent::Standby => self.tunnel_state = TunnelState::Standby,
            ReportSendingEvent::AllowDirectSending(status) => self.allow_direct_sending = status,
        }
        let new_value = self.allows_sending();
        if old_value == new_value {
            None
        } else {
            Some(new_value)
        }
    }
}

pub struct ReportSendingHandler {
    // Value to spawn sending task
    storage: StatsStorage,
    api_client: StatisticsApiClient,
    config: NetworkStatisticsConfig,

    inner_handler: Option<InnerHandlerHandle>,
}

impl ReportSendingHandler {
    pub fn new(
        storage: StatsStorage,
        config: NetworkStatisticsConfig,
        api_client: StatisticsApiClient,
    ) -> Self {
        let inner_handler = if config.enabled {
            Some(InnerHandler::start(
                storage.clone(),
                api_client.clone(),
                config.allow_disconnected,
            ))
        } else {
            None
        };

        Self {
            storage,
            api_client,
            config,
            inner_handler,
        }
    }

    pub async fn handle_command(&mut self, command: ConfigCommand) {
        match command {
            ConfigCommand::DisableCollection => {
                // Cancel sending task if it exists
                if let Some(handler) = self.inner_handler.take() {
                    tracing::debug!("Stats collection disabled : stopping sending task");
                    handler.cancellation_token.cancel();
                }
                self.config.enabled = false;
            }

            ConfigCommand::EnableCollection => {
                // Spawn sending task if it doesn't exist
                if self.inner_handler.is_none() {
                    self.inner_handler = Some(InnerHandler::start(
                        self.storage.clone(),
                        self.api_client.clone(),
                        self.config.allow_disconnected,
                    ));
                    tracing::debug!("Stats collection enabled : starting sending task");
                }
                self.config.enabled = true;
            }

            ConfigCommand::AllowDirectSending(status) => {
                self.config.allow_disconnected = status;
                // inform the sending task, if it's running
                if self.config.enabled {
                    self.handle_event(ReportSendingEvent::AllowDirectSending(status));
                }
            }
        }
    }

    // Cancel sending if it exists
    pub(crate) async fn on_shutdown(&mut self) {
        if let Some(handler) = self.inner_handler.take() {
            handler.cancellation_token.cancel();
        }
    }

    pub fn handle_event(&mut self, event: ReportSendingEvent) {
        // If we get an event, the handler should be running
        if let Some(handler) = &self.inner_handler
            && let Err(e) = handler.event_tx.try_send(event)
        {
            tracing::warn!("Failed to send event to report sending task : {e}");
        }
    }
}

struct InnerHandlerHandle {
    event_tx: Sender<ReportSendingEvent>,
    cancellation_token: CancellationToken,
}

struct InnerHandler {
    storage: StatsStorage,
    api_client: StatisticsApiClient,
    event_rx: Receiver<ReportSendingEvent>,
    sending_config: SendingConfig,
    system_report: StaticInformationReport,
    cancellation_token: CancellationToken,
}

impl InnerHandler {
    fn start(
        storage: StatsStorage,
        api_client: StatisticsApiClient,
        allow_direct_sending: bool,
    ) -> InnerHandlerHandle {
        let system_report = StaticInformationReport {
            os_type: System::distribution_id(),
            os_version: System::long_os_version(),
            os_arch: System::cpu_arch(),
            app_version: nym_bin_common::bin_info!().build_version.into(),
        };
        let sending_config = SendingConfig {
            tunnel_state: TunnelState::Disconnected,
            allow_direct_sending,
        };
        let (event_tx, event_rx) = mpsc::channel(10);
        let cancellation_token = CancellationToken::new();
        let inner_handler = Self {
            storage,
            api_client,
            event_rx,
            sending_config,
            system_report,
            cancellation_token: cancellation_token.clone(),
        };
        tokio::spawn(inner_handler.run());
        InnerHandlerHandle {
            event_tx,
            cancellation_token,
        }
    }

    async fn send_reports(
        storage: StatsStorage,
        api_client: StatisticsApiClient,
        system_report: StaticInformationReport,
        tunnel_state: TunnelState,
    ) -> Result<(), Error> {
        let reports_to_send = storage.get_pending_session_report_with_id().await?;
        let seed = storage.maybe_init_and_load_seed(None).await?;
        let identifier = generate_vpn_client_stats_id(seed);
        if reports_to_send.is_empty() {
            tracing::debug!("ReportHandler: Nothing to send");
            if tunnel_state == TunnelState::Connected {
                tracing::debug!("ReportHandler: Pinging active device");
                let report = ActiveDeviceReport::new(identifier, system_report);
                if let Err(e) = api_client.post_active_device(report).await {
                    tracing::warn!("Pinging active device failed : {e}");
                };
            }
            return Ok(());
        }

        tracing::debug!(
            "ReportHandler: Trying to send {} reports",
            reports_to_send.len()
        );

        let mut sending_errors = 0;

        for report_with_id in reports_to_send {
            let report = VpnClientStatsReportV2::new(
                identifier.clone(),
                system_report.clone(),
                report_with_id.report.into(),
            );
            if let Err(e) = api_client.post_session_report(report).await {
                tracing::error!(
                    "Failed to send session report with id {0} : {e}",
                    report_with_id.id
                );
                sending_errors += 1;
            } else {
                storage
                    .delete_pending_session_report(report_with_id.id)
                    .await.unwrap_or_else(|e| tracing::error!("Failed to delete session report with id {0}, it might be re-sent in the future : {e}", report_with_id.id))
            }
        }
        if sending_errors > 0 {
            tracing::debug!("Stats report handling ended with {sending_errors} errors");
            Err(Error::ReportSending)
        } else {
            Ok(())
        }
    }

    async fn run(mut self) {
        let mut sending_retries = 0;
        let sending_task = Fuse::terminated();
        let timer = if self.sending_config.allow_direct_sending {
            tokio::time::sleep(SendingConfig::random_small_delay()).fuse()
        } else {
            Fuse::terminated()
        };
        let mut timer = pin!(timer);
        let mut sending_task = pin!(sending_task);
        loop {
            tokio::select! {
                biased;
                _ = self.cancellation_token.cancelled() => {
                    // Sending will cancel itself because we're dropping the future
                    return;
                }
                maybe_event = self.event_rx.recv() => {
                    match maybe_event {
                        None => {
                            // Sending will cancel itself because we're dropping the future
                            return;
                        }
                        Some(event) => {
                            if let Some(new_allowed) = self.sending_config.update(event) {
                                // Handle policy changes
                                if new_allowed {
                                    let next_delay = SendingConfig::random_small_delay();
                                    tracing::debug!("Stats report conditions are met, trying to send in {} secs", next_delay.as_secs());
                                    timer.as_mut().set(tokio::time::sleep(next_delay).fuse());

                                } else {
                                    // We can't send, we need to abort
                                    timer.as_mut().set(tokio::time::sleep(SendingConfig::random_big_delay()).fuse());
                                    tracing::debug!("Stats report conditions not met, canceling sending task (if it was running)");
                                    sending_task.set(Fuse::terminated());
                                }
                            }

                        }
                    }
                }
                _ = &mut timer => {
                    sending_task.set(Self::send_reports(self.storage.clone(), self.api_client.clone(), self.system_report.clone(), self.sending_config.tunnel_state).fuse());
                }
                sending_res = &mut sending_task => {
                    sending_task.as_mut().set(Fuse::terminated());
                    match sending_res {

                        // Succesful sending
                        Ok(()) =>  {
                            let next_delay = SendingConfig::random_big_delay();
                            sending_retries = 0;
                            tracing::debug!("Stats report handling successful, next try in {} minutes", next_delay.as_secs() / 60);
                            timer.as_mut().set(tokio::time::sleep(next_delay).fuse());
                        },
                        // Sending error, retry
                        Err(Error::ReportSending) if sending_retries < SendingConfig::MAX_RETRIES => {
                            let next_delay = SendingConfig::random_small_delay();
                            sending_retries += 1;
                            tracing::error!("Stats report handling ended with errors, retrying in {} secs", next_delay.as_secs());
                            timer.as_mut().set(tokio::time::sleep(next_delay).fuse());
                        },
                        // Sending error no error
                        Err(Error::ReportSending) if sending_retries >= SendingConfig::MAX_RETRIES => {
                            let next_delay = SendingConfig::random_big_delay();
                            sending_retries = 0;
                            tracing::error!("Stats report handling ended with errors. Too many retries, retrying in {} mins", next_delay.as_secs() / 60);
                            timer.as_mut().set(tokio::time::sleep(next_delay).fuse());
                        },
                        // Storage error
                        Err(Error::StatsStorage(e)) => {
                            tracing::error!("Stats report handling ended with an unexpected storage error. We might not be able to send at all : {e}");
                            timer.as_mut().set(tokio::time::sleep(SendingConfig::random_big_delay()).fuse());
                        },
                        // Should never happen as of writing this
                        Err(e) => {
                            tracing::error!("Stats report handling ended with an error that shouldn't happen: {e}");
                            timer.as_mut().set(Fuse::terminated());
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        events::ReportSendingEvent,
        handler::report_sending::{SendingConfig, TunnelState},
    };

    fn mock_sending_config() -> SendingConfig {
        SendingConfig {
            tunnel_state: TunnelState::Disconnected,
            allow_direct_sending: false,
        }
    }

    #[tokio::test]
    async fn sending_policy_test() {
        let mut sending_policy = mock_sending_config();

        assert_eq!(sending_policy.update(ReportSendingEvent::Standby), None);
        assert_eq!(
            sending_policy.update(ReportSendingEvent::Connected),
            Some(true)
        );
        assert_eq!(
            sending_policy.update(ReportSendingEvent::Standby),
            Some(false)
        );
        assert_eq!(
            sending_policy.update(ReportSendingEvent::Disconnected),
            None
        );
        assert_eq!(
            sending_policy.update(ReportSendingEvent::AllowDirectSending(true)),
            Some(true)
        );
        assert_eq!(
            sending_policy.update(ReportSendingEvent::AllowDirectSending(false)),
            Some(false)
        );
    }
}
