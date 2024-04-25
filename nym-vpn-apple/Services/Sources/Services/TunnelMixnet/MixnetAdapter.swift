import Foundation
import Constants
import NetworkExtension
import MixnetLibrary

public final class MixnetAdapter {
    private weak var packetTunnelProvider: NEPacketTunnelProvider?

    public let mixnetTunnelProvider: MixnetTunnelProvider

    public var tunnelFileDescriptor: Int32? {
        var buf = [CChar](repeating: 0, count: Int(IFNAMSIZ))
        for fd: Int32 in 0...1024 {
            var len = socklen_t(buf.count)
            if getsockopt(fd, 2, 2, &buf, &len) == 0 && String(cString: buf).hasPrefix("utun") {
                return fd
            }
        }
        return nil
    }

    public init(
        with packetTunnelProvider: NEPacketTunnelProvider,
        mixnetTunnelProvider: MixnetTunnelProvider
    ) {
        self.packetTunnelProvider = packetTunnelProvider
        self.mixnetTunnelProvider = mixnetTunnelProvider
        setup()
    }

    public func start(with vpnConfig: VpnConfig) throws {
        do {
            try runVpn(config: vpnConfig)
        } catch let error {
            throw error
        }
    }

    public func stop() throws {
        do {
            try stopVpn()
        } catch let error {
            throw error
        }
    }
}

private extension MixnetAdapter {
    func setup() {
        setupEnvironmentVariables()
    }

    func setupEnvironmentVariables() {
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

        setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "https://mainnet-stats.nymte.ch:8090", 1)
        setenv("EXPLORER_API", Constants.explorerUrl.rawValue, 1)
        setenv("NYXD", "https://rpc.nymtech.net", 1)
        setenv("NYXD_WS", "wss://rpc.nymtech.net/websocket", 1)
        setenv("NYM_API", Constants.apiUrl.rawValue, 1)
    }

    func setupSandboxEnvironmentVariables() {
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

        setenv("REWARDING_VALIDATOR_ADDRESS", "n1pefc2utwpy5w78p2kqdsfmpjxfwmn9d39k5mqa", 1)
        setenv("MIXNET_CONTRACT_ADDRESS", "n1xr3rq8yvd7qplsw5yx90ftsr2zdhg4e9z60h5duusgxpv72hud3sjkxkav", 1)
        setenv("VESTING_CONTRACT_ADDRESS", "n1unyuj8qnmygvzuex3dwmg9yzt9alhvyeat0uu0jedg2wj33efl5qackslz", 1)
        setenv("COCONUT_BANDWIDTH_CONTRACT_ADDRESS", "n13902g92xfefeyzuyed49snlm5fxv5ms6mdq5kvrut27hasdw5a9q9vyw6c", 1)
        setenv("GROUP_CONTRACT_ADDRESS", "n18nczmqw6adwxg2wnlef3hf0etf8anccafp2pjpul5rrtmv96umyq5mv7t5", 1)
        setenv("MULTISIG_CONTRACT_ADDRESS", "n1q3zzxl78rlmxv3vn0uf4vkyz285lk8q2xzne299yt9x6mpfgk90qukuzmv", 1)
        setenv("COCONUT_DKG_CONTRACT_ADDRESS", "n1jsz20ggp5a6v76j060erkzvxmeus8htlpl77yxp878f0gf95cyaq6p2pee", 1)
        setenv("NAME_SERVICE_CONTRACT_ADDRESS", "n12ne7qtmdwd0j03t9t5es8md66wq4e5xg9neladrsag8fx3y89rcs36asfp", 1)
        setenv(
            "SERVICE_PROVIDER_DIRECTORY_CONTRACT_ADDRESS",
            "n1ps5yutd7sufwg058qd7ac7ldnlazsvmhzqwucsfxmm445d70u8asqxpur4",
            1
        )
        setenv("EPHEMERA_CONTRACT_ADDRESS", "n19lc9u84cz0yz3fww5283nucc9yvr8gsjmgeul0", 1)

        setenv("STATISTICS_SERVICE_DOMAIN_ADDRESS", "http://0.0.0.0", 1)
        setenv("EXPLORER_API", Constants.sandboxExplorerUrl.rawValue, 1)
        setenv("NYXD", "https://rpc.sandbox.nymtech.net", 1)
        setenv("NYXD_WS", "wss://rpc.sandbox.nymtech.net/websocket", 1)
        setenv("NYM_API", Constants.sandboxApiUrl.rawValue, 1)
    }
}
