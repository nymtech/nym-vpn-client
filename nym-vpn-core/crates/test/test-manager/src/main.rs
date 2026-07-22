// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod config;
#[cfg(target_os = "linux")]
mod container;
mod device_cleanup;
mod logging;
mod nym_daemon;
mod run_tests;
mod summary;
mod tests;
mod vm;

use anyhow::{Context, Ok, Result};
use clap::Parser;
use config::ConfigFile;
#[cfg(target_os = "macos")]
use std::net::IpAddr;
use std::path::PathBuf;
use tests::{config_nym::TEST_CONFIG_NYM, get_filtered_tests};
use vm::provision;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    cmd: Commands,
}

#[derive(clap::Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Manage configuration for tests and VMs
    #[clap(subcommand)]
    Config(ConfigArg),

    /// Spawn a runner instance without running any tests
    RunVm {
        /// Name of the VM config, run `test-manager config vm list` to see configured VMs
        vm: String,

        /// Run VNC server on a specified port
        #[arg(long)]
        vnc: Option<u16>,

        /// Make permanent changes to image
        #[arg(long)]
        keep_changes: bool,
    },

    /// List all tests and their priority.
    ListTests,

    /// Spawn a runner instance and run tests
    RunTests {
        /// Name of the VM config, run `test-manager config vm list` to see configured VMs
        #[arg(long)]
        vm: String,

        /// Show display of guest
        #[arg(long, group = "display_args")]
        display: bool,

        /// Make permanent changes to image
        #[arg(long)]
        keep_changes: bool,

        /// Run VNC server on a specified port
        #[arg(long, group = "display_args")]
        vnc: Option<u16>,

        /// mnemonic in the form of 24 words to use in testing
        #[arg(long)]
        nym_mnemonic: String,

        /// Names of tests to not run. Any tests given here will be skipped.
        #[arg(long)]
        skip: Vec<String>,

        /// Names of tests to run. The order given will be respected. If not set, all tests will be
        /// run.
        test_filters: Vec<String>,

        /// Print results live
        #[arg(long, short)]
        verbose: bool,

        /// Path to output test results in a structured format
        #[arg(long, value_name = "PATH")]
        test_report: Option<PathBuf>,

        /// Path to the directory containing the test runner
        #[arg(long, value_name = "DIR")]
        runner_dir: Option<PathBuf>,
    },

    /// Output an HTML-formatted summary of one or more reports
    FormatTestReports {
        /// One or more test reports output by 'test-manager run-tests --test-report'
        reports: Vec<PathBuf>,
    },

    /// Delete all devices registered on the test account (pre-run cleanup)
    DeleteDevices {
        /// Account mnemonic (24 words)
        #[arg(long)]
        nym_mnemonic: String,
    },

    /// Update the system image
    ///
    /// Note that in order for the updates to take place, the VM's config need
    /// to have `provisioner` set to `ssh`, `ssh_user` & `ssh_password` set and
    /// the `ssh_user` should be able to execute commands with sudo/ as root.
    Update {
        /// Name of the VM config
        name: String,
    },
}

#[derive(clap::Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
enum ConfigArg {
    /// Print the current config
    Get,
    /// Print the path to the current config file
    Which,
    /// Manage VM-specific setting
    #[clap(subcommand)]
    Vm(VmConfig),
}

#[derive(clap::Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
enum VmConfig {
    /// Create or edit a VM config
    Set {
        /// Name of the VM config
        vm: String,

        /// VM config
        #[clap(flatten)]
        config: config::VmConfig,
    },

    /// Remove specified VM config
    Remove {
        /// Name of the VM config, run `test-manager config vm list` to see configured VMs
        vm: String,
    },

    /// List available VM configurations
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    logging::Logger::get_or_init();

    let args = Args::parse();

    let mut config = config::ConfigFile::load_or_default()
        .await
        .context("Failed to load config")?;
    match args.cmd {
        Commands::Config(config_subcommand) => match config_subcommand {
            ConfigArg::Get => {
                println!("{:#?}", *config);
                Ok(())
            }
            ConfigArg::Which => {
                println!(
                    "{}",
                    ConfigFile::get_config_path()
                        .expect("Get config path")
                        .display()
                );
                Ok(())
            }
            ConfigArg::Vm(vm_config) => match vm_config {
                VmConfig::Set {
                    vm,
                    config: vm_config,
                } => vm::set_config(&mut config, &vm, vm_config)
                    .await
                    .context("Failed to edit or create VM config"),
                VmConfig::Remove { vm } => {
                    if config.get_vm(&vm).is_none() {
                        println!("No such configuration");
                        return Ok(());
                    }
                    config
                        .edit(|config| {
                            config.vms.remove_entry(&vm);
                        })
                        .await
                        .context("Failed to remove config entry")?;
                    println!("Removed configuration \"{vm}\"");
                    Ok(())
                }
                VmConfig::List => {
                    println!("Configured VMs:");
                    for vm in config.vms.keys() {
                        println!("{vm}");
                    }
                    Ok(())
                }
            },
        },
        Commands::ListTests => {
            println!("priority\tname");
            for test in tests::get_test_descriptions() {
                println!(
                    "{priority:8}\t{name}",
                    name = test.name,
                    priority = test.priority.unwrap_or(0),
                );
            }
            Ok(())
        }
        Commands::FormatTestReports { reports } => {
            summary::print_summary_table(&reports).await;
            Ok(())
        }
        Commands::DeleteDevices { nym_mnemonic } => {
            device_cleanup::delete_all_devices(&nym_mnemonic)
                .await
                .context("Failed to delete account devices")?;
            Ok(())
        }
        Commands::RunVm {
            vm,
            vnc,
            keep_changes,
        } => {
            #[cfg(target_os = "linux")]
            container::relaunch_with_rootlesskit(vnc).await;

            let mut config = config.clone();
            config.runtime_opts.keep_changes = keep_changes;
            config.runtime_opts.display = if vnc.is_some() {
                config::Display::Vnc
            } else {
                config::Display::Local
            };
            log::info!("🖥️ Display mode: {:?}", &config.runtime_opts.display);

            let mut instance = vm::run(&config, &vm).await.context("Failed to start VM")?;

            instance.wait().await;

            Ok(())
        }
        Commands::RunTests {
            vm,
            display,
            keep_changes,
            vnc,
            nym_mnemonic,
            skip,
            test_filters,
            verbose,
            test_report,
            runner_dir,
        } => {
            #[cfg(target_os = "linux")]
            container::relaunch_with_rootlesskit(vnc).await;

            let mut config = config.clone();
            // TODO this keeps changes manually because there is no CLI flag for it under run-tests
            config.runtime_opts.keep_changes = keep_changes;
            log::info!("Keep changes? {keep_changes}");
            config.runtime_opts.display = match (display, vnc.is_some()) {
                (false, false) => config::Display::None,
                (true, false) => config::Display::Local,
                (false, true) => config::Display::Vnc,
                (true, true) => unreachable!("invalid combination"),
            };
            log::info!("🖥️ Display mode: {:?}", &config.runtime_opts.display);

            let vm_config = vm::get_vm_config(&config, &vm).context("Cannot get VM config")?;

            let mut instance = vm::run(&config, &vm).await.context("Failed to start VM")?;
            let runner_dir = runner_dir.unwrap_or_else(|| {
                log::debug!("Using default runner dir");
                vm_config.get_default_runner_dir()
            });
            log::debug!("Runner dir: {}", &runner_dir.display());
            let artifacts_dir = provision::provision(vm_config, &*instance, runner_dir)
                .await
                .context("Failed to run provisioning for VM")?;

            #[cfg(target_os = "macos")]
            let IpAddr::V4(guest_ip) = instance.get_ip().to_owned() else {
                panic!("Expected bridge IP to be version 4, but was version 6.")
            };
            let (bridge_name, bridge_ip) = vm::network::bridge(
                #[cfg(target_os = "macos")]
                &guest_ip,
            )?;
            log::debug!("bridge_name: {}, bridge_ip: {}", bridge_name, bridge_ip);

            TEST_CONFIG_NYM.init(tests::config_nym::TestConfigNym::new(
                nym_mnemonic,
                artifacts_dir,
                bridge_name,
                bridge_ip,
                test_rpc::meta::Os::from(vm_config.os_type),
            ));

            let mut tests = get_filtered_tests(&test_filters, &skip)?;
            log::info!("{} filtered tests:\n{:#?}", tests.len(), &tests);

            for test in tests.iter_mut() {
                test.location = config.test_locations.lookup(test.name).cloned();
            }

            // For convenience, spawn a SOCKS5 server that is reachable for tests that need it
            // let socks = socks_server::spawn(SocketAddr::new(
            //     TEST_CONFIG.host_bridge_ip.into(),
            //     crate::vm::network::SOCKS5_PORT,
            // ))
            // .await?;

            let skip_wait = vm_config.provisioner != config::Provisioner::Noop;

            let summary_logger = match test_report {
                Some(path) => Some(
                    summary::SummaryLogger::new(
                        &vm,
                        test_rpc::meta::Os::from(vm_config.os_type),
                        &path,
                    )
                    .await
                    .context("Failed to create summary logger")?,
                ),
                None => None,
            };

            let result = run_tests::run(&*instance, tests, skip_wait, !verbose, summary_logger)
                .await
                .context("Tests failed");

            if display {
                instance.wait().await;
            }
            // socks.close();
            // Propagate any error from the test run if applicable
            result?.anyhow()
        }
        Commands::Update { name } => {
            let vm_config = vm::get_vm_config(&config, &name).context("Cannot get VM config")?;

            let instance = vm::run(&config, &name)
                .await
                .context("Failed to start VM")?;

            let update_output = vm::update_packages(vm_config.clone(), &*instance)
                .await
                .context("Failed to update packages to the VM image")?;
            log::info!("Update command finished with output: {}", &update_output);
            // TODO: If the update was successful, commit the changes to the VM image.
            log::info!("Note: updates have not been persisted to the image");
            Ok(())
        }
    }
}
