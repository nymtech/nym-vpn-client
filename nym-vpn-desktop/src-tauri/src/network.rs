use std::path::PathBuf;

use anyhow::{Context, Result};
use nym_vpn_lib::nym_config;
use tokio::fs::read;
use tracing::{debug, error, info, instrument};

#[instrument]
pub async fn setup_network_env(use_sandbox: bool, env_config_file: &Option<PathBuf>) -> Result<()> {
    if use_sandbox {
        info!("network environment: sandbox");
        // TODO: instead bundle a sandbox env file we can use
        setup_sandbox_environment();
        return Ok(());
    }

    if let Some(file) = env_config_file {
        debug!("provided env_config_file: {}", file.display());

        // check if the file exists and is readable
        read(file)
            .await
            .inspect_err(|e| error!("failed to read `env_config_file`: {e}"))
            .context(format!(
                "app config, failed to read the provided `env_config_file`: `{}`",
                file.display()
            ))?;

        info!("network environment: custom env {}", file.display());
        nym_config::defaults::setup_env(env_config_file.clone());
        return Ok(());
    }

    info!("network environment: mainnet");
    nym_config::defaults::setup_env::<PathBuf>(None);

    Ok(())
}

// Setup sandbox environment. This is tempory until we switch to mainnet at which point we can
// purge this function
fn setup_sandbox_environment() {
    debug!("setting up builtin sandbox environment");

    std::env::set_var("CONFIGURED", "true");
    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    std::env::set_var("NETWORK_NAME", "sandbox");
    std::env::set_var("BECH32_PREFIX", "n");
    std::env::set_var("MIX_DENOM", "unym");
    std::env::set_var("MIX_DENOM_DISPLAY", "nym");
    std::env::set_var("STAKE_DENOM", "unyx");
    std::env::set_var("STAKE_DENOM_DISPLAY", "nyx");
    std::env::set_var("DENOMS_EXPONENT", "6");
    std::env::set_var(
        "REWARDING_VALIDATOR_ADDRESS",
        "n1pefc2utwpy5w78p2kqdsfmpjxfwmn9d39k5mqa",
    );
    std::env::set_var(
        "MIXNET_CONTRACT_ADDRESS",
        "n1xr3rq8yvd7qplsw5yx90ftsr2zdhg4e9z60h5duusgxpv72hud3sjkxkav",
    );
    std::env::set_var(
        "VESTING_CONTRACT_ADDRESS",
        "n1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qackslz",
    );
    std::env::set_var(
        "COCONUT_BANDWIDTH_CONTRACT_ADDRESS",
        "n13902g92xfefeyzuyed49snlm5fxv5ms6mdq5kvrut27hasdw5a9q9vyw6c",
    );
    std::env::set_var(
        "GROUP_CONTRACT_ADDRESS",
        "n18nczmqw6adwxg2wnlef3hf0etf8anccafp2pjpul5rrtmv96umyq5mv7t5",
    );
    std::env::set_var(
        "MULTISIG_CONTRACT_ADDRESS",
        "n1q3zzxl78rlmxv3vn0uf4vkyz285lk8q2xzne299yt9x6mpfgk90qukuzmv",
    );
    std::env::set_var(
        "COCONUT_DKG_CONTRACT_ADDRESS",
        "n1jsz20ggp5a6v76j060erkzvxmeus8htlpl77yxp878f0gf95cyaq6p2pee",
    );
    std::env::set_var(
        "NAME_SERVICE_CONTRACT_ADDRESS",
        "n12ne7qtmdwd0j03t9t5es8md66wq4e5xg9neladrsag8fx3y89rcs36asfp",
    );
    std::env::set_var(
        "SERVICE_PROVIDER_DIRECTORY_CONTRACT_ADDRESS",
        "n1ps5yutd7sufwg058qd7ac7ldnlazsvmhzqwucsfxmm445d70u8asqxpur4",
    );
    std::env::set_var(
        "EPHEMERA_CONTRACT_ADDRESS",
        "n19lc9u84cz0yz3fww5283nucc9yvr8gsjmgeul0",
    );
    std::env::set_var("STATISTICS_SERVICE_DOMAIN_ADDRESS", "http://0.0.0.0");
    std::env::set_var("EXPLORER_API", "https://sandbox-explorer.nymtech.net/api");
    std::env::set_var("NYXD", "https://rpc.sandbox.nymtech.net");
    std::env::set_var("NYXD_WS", "wss://rpc.sandbox.nymtech.net/websocket");
    std::env::set_var("NYM_API", "https://sandbox-nym-api1.nymtech.net/api");
}
