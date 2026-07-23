// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::vm::ssh::SSHSession;
use crate::{
    logging::{Logger, Panic, TestOutput, TestResult},
    nym_daemon::{self, RpcClientProvider},
    summary::SummaryLogger,
    tests::{TestContext, TestMetadata, TestWrapperFunctionNym, helpers_nym},
    vm,
};
use anyhow::{Context, Result};
use futures::FutureExt;
use nym_vpn_proto::rpc_client::RpcClient as NymProxyClient;
use std::{io::Write, net::IpAddr, panic, time::Duration};
use test_rpc::{client_nym::NymServiceClient, logging::Output};

const GUEST_DAEMON_LOG_PATH: &str = "/var/log/nym-vpnd/nym-vpnd.log";

const DAEMON_LOG_TAIL_LINES: usize = 4000;

#[derive(Clone)]
struct GuestSshAccess {
    user: String,
    password: String,
    ip: IpAddr,
}

/// Fetch the daemon log over SSH, retrying briefly since the guest may be
/// firewalled off by the kill-switch during a connect attempt.
async fn fetch_guest_daemon_log(access: GuestSshAccess) -> Result<String> {
    tokio::task::spawn_blocking(move || {
        let mut last_err = None;
        for attempt in 1..=5 {
            match SSHSession::connect(access.user.clone(), access.password.clone(), access.ip)
                .and_then(|session| {
                    session.exec_blocking(&format!("sudo cat {GUEST_DAEMON_LOG_PATH}"))
                }) {
                Ok(contents) if !contents.trim().is_empty() => return Ok(contents),
                Ok(_) => {
                    log::debug!(
                        "daemon log capture attempt {attempt}/5: empty output (missing/unreadable file?)"
                    );
                    last_err = Some(anyhow::anyhow!("empty daemon log output"));
                    std::thread::sleep(Duration::from_secs(3));
                }
                Err(err) => {
                    log::debug!("daemon log capture attempt {attempt}/5 failed: {err:#}");
                    last_err = Some(err);
                    std::thread::sleep(Duration::from_secs(3));
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("no attempts made")))
    })
    .await
    .context("daemon log capture task panicked")?
}

/// Best-effort: write the full daemon log to a host file and echo the tail into
/// the CI output. Never fails the run.
///
/// The guest file is cumulative across the suite (one daemon process, one log).
/// Named artifacts are snapshots of that file at failure time, not per-test sessions.
async fn capture_daemon_log_on_failure(access: &GuestSshAccess, test_name: &str) {
    let contents = match fetch_guest_daemon_log(access.clone()).await {
        Ok(contents) => contents,
        Err(err) => {
            log::warn!("Could not capture daemon log for '{test_name}' over SSH: {err:#}");
            return;
        }
    };

    let first_ts = contents
        .lines()
        .find(|line| !line.trim().is_empty())
        .and_then(daemon_log_line_timestamp)
        .unwrap_or("unknown");
    let last_ts = contents
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .and_then(daemon_log_line_timestamp)
        .unwrap_or("unknown");
    let line_count = contents.lines().count();

    let preamble = format!(
        "# cumulative guest log ({GUEST_DAEMON_LOG_PATH}); not a per-test session\n\
         # captured_for_test={test_name}\n\
         # first_ts={first_ts} last_ts={last_ts} lines={line_count}\n"
    );

    // Write to /tmp so CI can collect it (same place as the test report), rather
    // than the ephemeral working dir.
    let file_name = format!("/tmp/daemon-log-{test_name}.log");
    let write_result = (|| -> std::io::Result<()> {
        let mut f = std::fs::File::create(&file_name)?;
        f.write_all(preamble.as_bytes())?;
        f.write_all(contents.as_bytes())?;
        Ok(())
    })();
    match write_result {
        Ok(()) => log::info!(
            "Wrote full daemon log for '{test_name}' to {file_name} (cumulative; first_ts={first_ts} last_ts={last_ts} lines={line_count})"
        ),
        Err(err) => log::warn!("Failed to write daemon log file {file_name}: {err}"),
    }

    let lines: Vec<&str> = contents.lines().collect();
    let tail = lines
        .iter()
        .skip(lines.len().saturating_sub(DAEMON_LOG_TAIL_LINES))
        .copied()
        .collect::<Vec<_>>()
        .join("\n");
    log::info!(
        "===== nym-vpnd log tail ({} of {} lines) for {test_name} =====\n\
         (cumulative guest log; first_ts={first_ts} last_ts={last_ts})\n\
         {tail}\n===== end nym-vpnd log for {test_name} =====",
        DAEMON_LOG_TAIL_LINES.min(lines.len()),
        lines.len(),
    );
}

fn daemon_log_line_timestamp(line: &str) -> Option<&str> {
    // Typical format: 2026-07-23T06:07:11.835343Z  INFO ...
    let ts = line.split_whitespace().next()?;
    if ts.len() >= 20 && ts.contains('T') {
        Some(ts)
    } else {
        None
    }
}

/// The baud rate of the serial connection between the test manager and the test runner.
/// There is a known issue with setting a baud rate at all or macOS, and the workaround
/// is to set it to zero: https://github.com/serialport/serialport-rs/pull/58
///
/// Keep this constant in sync with `test-runner/src/main.rs`
const BAUD: u32 = if cfg!(target_os = "macos") { 0 } else { 115200 };

struct TestHandler<'a> {
    rpc_provider: &'a RpcClientProvider,
    test_runner_client: &'a NymServiceClient,
    failed_tests: Vec<&'static str>,
    successful_tests: Vec<&'static str>,
    summary_logger: Option<SummaryLogger>,
    print_failed_tests_only: bool,
    logger: Logger,
    /// SSH access for out-of-band daemon log capture on failure.
    guest_ssh_access: Option<GuestSshAccess>,
}

impl TestHandler<'_> {
    async fn run_test(
        &mut self,
        test: TestWrapperFunctionNym,
        test_name: &'static str,
        nym_client: Option<NymProxyClient>,
    ) -> Result<(), anyhow::Error> {
        log::info!("▶️ Running {test_name}");

        if self.print_failed_tests_only {
            // Stop live record
            self.logger.store_records(true);
        }

        let test_output = run_test_function(
            self.test_runner_client.clone(),
            nym_client,
            test,
            test_name,
            TestContext {
                _rpc_provider: self.rpc_provider.clone(),
            },
        )
        .await;

        if self.print_failed_tests_only {
            // Print results of failed test
            if test_output.result.failure() {
                self.logger.print_stored_records();
            } else {
                self.logger.flush_records();
            }
            self.logger.store_records(false);
        }

        if test_output.result.failure() {
            // Test consumed its NymProxyClient; open a fresh one to clear a leftover tunnel.
            if let Ok(mut client) = self.rpc_provider.new_client_nym().await {
                helpers_nym::best_effort_disconnect(&mut client).await;
            } else {
                log::warn!(
                    "Could not create RPC client for fail-path disconnect after '{test_name}'"
                );
            }

            if let Some(access) = &self.guest_ssh_access {
                capture_daemon_log_on_failure(access, test_name).await;
            }
        }

        test_output.print();

        register_test_result(
            test_output.result,
            &mut self.failed_tests,
            test_name,
            &mut self.successful_tests,
            self.summary_logger.as_mut(),
        )
        .await?;

        Ok(())
    }

    fn gather_results(self) -> TestResult {
        log::info!("TESTS THAT SUCCEEDED:");
        for test in self.successful_tests {
            log::info!("{test}");
        }

        log::info!("TESTS THAT FAILED:");
        for test in &self.failed_tests {
            log::info!("{test}");
        }

        if self.failed_tests.is_empty() {
            TestResult::Pass
        } else {
            TestResult::Fail(anyhow::anyhow!("Some tests failed"))
        }
    }
}

pub async fn run(
    instance: &dyn vm::VmInstance,
    tests: Vec<TestMetadata>,
    skip_wait: bool,
    print_failed_tests_only: bool,
    summary_logger: Option<SummaryLogger>,
    ssh_options: Option<(String, String)>,
) -> Result<TestResult> {
    let guest_ssh_access = ssh_options.map(|(user, password)| GuestSshAccess {
        user,
        password,
        ip: *instance.get_ip(),
    });
    log::trace!("Setting test constants");

    let pty_path = instance.get_pty();

    log::debug!("Connecting to PTY at {pty_path}");

    let timeout = Duration::from_secs(60 * 5);
    // one serial connection for all communications with the test-runner
    let serial_stream = tokio::time::timeout(timeout, async {
        loop {
            match tokio_serial::SerialStream::open(&tokio_serial::new(pty_path, BAUD)) {
                Ok(stream) => break stream,
                Err(err) => {
                    log::warn!("{err}, retrying...");
                    tokio::time::sleep(Duration::from_secs(15)).await;
                    continue;
                }
            }
        }
    })
    .await?;
    // runner transport - for tarpc test RPC calls to test-rpc
    // daemon transport - for gRPC calls to Nym daemon
    let (runner_transport, daemon_transport, mut connection_handle, completion_handle) =
        test_rpc::transport::create_client_transports(serial_stream)?;

    if !skip_wait {
        connection_handle.wait_for_server().await?;
    }

    log::info!("Running client");

    let nym_service_client = NymServiceClient::new(connection_handle.clone(), runner_transport);
    log::debug!("Created service client");
    let rpc_provider = nym_daemon::new_rpc_client(connection_handle, daemon_transport);
    log::debug!("Created RPC client to nym-vpn daemon");

    // TODO dz uncomment, make work with Nym
    // print_os_version(&test_runner_client).await;

    let mut test_handler = TestHandler {
        rpc_provider: &rpc_provider,
        test_runner_client: &nym_service_client,
        failed_tests: vec![],
        successful_tests: vec![],
        summary_logger,
        print_failed_tests_only,
        logger: Logger::get_or_init(),
        guest_ssh_access,
    };

    for test in tests {
        let nym_client = rpc_provider.new_client_nym().await?;

        test_handler
            .run_test(test.func, test.name, Some(nym_client))
            .await?;
    }

    let result = test_handler.gather_results();

    // runs unconditionally after all tests, outside the test loop, so it isn't
    // affected by test filtering or priority sorting.
    // Fixes "device leak": if we don't do this, devices registered on the
    // account pile up and we end up with MaxDevicesReached
    deregister_account(&rpc_provider).await;

    // wait for cleanup
    drop(rpc_provider);
    let _ = tokio::time::timeout(Duration::from_secs(5), completion_handle).await;

    Ok(result)
}

async fn deregister_account(rpc_provider: &RpcClientProvider) {
    log::info!("Cleaning up: forget_account to deregister device...");
    match rpc_provider.new_client_nym().await {
        Ok(mut nym_client) => {
            if let Err(e) = nym_client.forget_account().await {
                log::warn!("Failed to forget account during cleanup: {e}");
            } else {
                log::info!("Device deregistered successfully");
            }
        }
        Err(e) => {
            log::warn!("Failed to create RPC client for cleanup: {e}");
        }
    }
}

async fn register_test_result(
    test_result: TestResult,
    failed_tests: &mut Vec<&str>,
    test_name: &'static str,
    successful_tests: &mut Vec<&str>,
    summary_logger: Option<&mut SummaryLogger>,
) -> anyhow::Result<()> {
    if let Some(logger) = summary_logger {
        logger
            .log_test_result(test_name, test_result.summary())
            .await
            .context("Failed to log test result")?
    };

    if test_result.failure() {
        failed_tests.push(test_name);
    } else {
        successful_tests.push(test_name);
    }

    Ok(())
}

pub async fn run_test_function(
    runner_rpc: NymServiceClient,
    proxy_rpc: Option<NymProxyClient>,
    test: TestWrapperFunctionNym,
    test_name: &'static str,
    test_context: super::tests::TestContext,
) -> TestOutput {
    let _flushed = runner_rpc.try_poll_output().await;

    // Assert that the test is unwind safe, this is the same assertion that cargo tests do. This
    // assertion being incorrect can not lead to memory unsafety however it could theoretically
    // lead to logic bugs. The problem of forcing the test to be unwind safe is that it causes a
    // large amount of unergonomic design.
    let result: TestResult =
        panic::AssertUnwindSafe(test(test_context, runner_rpc.clone(), proxy_rpc))
            .catch_unwind()
            .await
            .map_err(Panic::new)
            .into();

    let mut output = vec![];
    if result.failure() {
        let output_after_test = runner_rpc.try_poll_output().await;
        match output_after_test {
            Ok(mut output_after_test) => {
                output.append(&mut output_after_test);
            }
            Err(e) => {
                output.push(Output::Other(format!("could not get logs: {e:?}")));
            }
        }
    }

    let log_output = runner_rpc.get_nym_app_logs().await.ok();

    TestOutput {
        log_output,
        test_name,
        error_messages: output,
        result,
    }
}

#[cfg(test)]
mod daemon_log_tests {
    use super::daemon_log_line_timestamp;

    #[test]
    fn parses_tracing_timestamp() {
        let line = "2026-07-23T06:07:11.835343Z  INFO nym_vpnd: Connected";
        assert_eq!(
            daemon_log_line_timestamp(line),
            Some("2026-07-23T06:07:11.835343Z")
        );
    }

    #[test]
    fn rejects_non_timestamp_lines() {
        assert_eq!(daemon_log_line_timestamp("# cumulative guest log"), None);
        assert_eq!(daemon_log_line_timestamp(""), None);
        assert_eq!(daemon_log_line_timestamp("INFO only"), None);
    }
}
