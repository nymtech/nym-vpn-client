#if os(iOS)
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
        setenv("EXPLORER_API", Constants.explorerURL.rawValue, 1)
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
#endif
