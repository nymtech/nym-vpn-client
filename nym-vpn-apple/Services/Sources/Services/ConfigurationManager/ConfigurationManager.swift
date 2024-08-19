import Foundation
import Constants

public final class ConfigurationManager {
    public static func configureMainnetEnvironmentVariables() {
        setenv("CONFIGURED", "true", 1)
        setenv("RUST_LOG", "info", 1)
        setenv("RUST_BACKTRACE", "1", 1)
        setenv("NETWORK_NAME", "mainnet", 1)
        setenv("BECH32_PREFIX", "n", 1)
        setenv("MIX_DENOM", "unym", 1)
        setenv("MIX_DENOM_DISPLAY", "nym", 1)
        setenv("STAKE_DENOM", "unyx", 1)
        setenv("STAKE_DENOM_DISPLAY", "nyx", 1)
        setenv("DENOMS_EXPONENT", "6", 1)

        setenv("REWARDING_VALIDATOR_ADDRESS", "n10yyd98e2tuwu0f7ypz9dy3hhjw7v772q6287gy", 1)
        setenv("MIXNET_CONTRACT_ADDRESS", "n17srjznxl9dvzdkpwpw24gg668wc73val88a6m5ajg6ankwvz9wtst0cznr", 1)
        setenv("VESTING_CONTRACT_ADDRESS", "n1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrq73f2nw", 1)
        setenv("GROUP_CONTRACT_ADDRESS", "n1e2zq4886zzewpvpucmlw8v9p7zv692f6yck4zjzxh699dkcmlrfqk2knsr", 1)
        setenv("MULTISIG_CONTRACT_ADDRESS", "n1txayqfz5g9qww3rlflpg025xd26m9payz96u54x4fe3s2ktz39xqk67gzx", 1)
        setenv("COCONUT_DKG_CONTRACT_ADDRESS", "n19604yflqggs9mk2z26mqygq43q2kr3n932egxx630svywd5mpxjsztfpvx", 1)

        setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "https://mainnet-stats.nymte.ch:8090", 1)
        setenv("EXPLORER_API", Constants.explorerURL.rawValue, 1)
        setenv("NYXD", "https://rpc.nymtech.net", 1)
        setenv("NYXD_WS", "wss://rpc.nymtech.net/websocket", 1)
        setenv("NYM_API", Constants.apiUrl.rawValue, 1)
        setenv("NYM_VPN_API", "https://nymvpn.com/api", 1)
    }

    public static func configureSandboxEnvironmentVariables() {
        setenv("CONFIGURED", "true", 1)
        setenv("RUST_LOG", "info", 1)
        setenv("RUST_BACKTRACE", "1", 1)
        setenv("NETWORK_NAME", "sandbox", 1)
        setenv("BECH32_PREFIX", "n", 1)
        setenv("MIX_DENOM", "unym", 1)
        setenv("MIX_DENOM_DISPLAY", "nym", 1)
        setenv("STAKE_DENOM", "unyx", 1)
        setenv("STAKE_DENOM_DISPLAY", "nyx", 1)
        setenv("DENOMS_EXPONENT", "6", 1)

        setenv("REWARDING_VALIDATOR_ADDRESS", "n1duuyj2th2y0z4u4f4wtljpdz9s3pxtu0xx6zdz", 1)
        setenv("MIXNET_CONTRACT_ADDRESS", "n14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sjyvg3g", 1)
        setenv("COCONUT_BANDWIDTH_CONTRACT_ADDRESS", "n1mf6ptkssddfmxvhdx0ech0k03ktp6kf9yk59renau2gvht3nq2gqt5tdrk", 1)
        setenv("GROUP_CONTRACT_ADDRESS", "n1qg5ega6dykkxc307y25pecuufrjkxkaggkkxh7nad0vhyhtuhw3sa07c47", 1)
        setenv("MULTISIG_CONTRACT_ADDRESS", "n1zwv6feuzhy6a9wekh96cd57lsarmqlwxdypdsplw6zhfncqw6ftqx5a364", 1)
        setenv("COCONUT_DKG_CONTRACT_ADDRESS", "n1aakfpghcanxtc45gpqlx8j3rq0zcpyf49qmhm9mdjrfx036h4z5sy2vfh9", 1)

        setenv("EXPLORER_API", Constants.sandboxExplorerURL.rawValue, 1)
        setenv("NYXD", "https://canary-validator.performance.nymte.ch", 1)
        setenv("NYM_API", Constants.sandboxApiUrl.rawValue, 1)
    }
}
