import Foundation
import AppSettings
import ConnectionTypes
import Tunnels
#if os(macOS)
import GRPCManager
#endif

@MainActor
extension ConnectionManager {
    public func setGatewayIndependenceNotifications(_ enabled: Bool) {
        appSettings.serverFamilyRemindersEnabled = enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setGatewayIndependenceNotifications(enabled))
#elseif os(macOS)
            try? await grpcManager.setGatewayIndependenceNotifications(enabled)
#endif
        }
    }

    public func setCustomDns(_ dns: [String]) {
        appSettings.customDns = dns
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setCustomDns(dns))
#elseif os(macOS)
            try? await grpcManager.setCustomDns(dnsServers: dns)
#endif
        }
    }

    public func setCustomDnsEnabled(_ enabled: Bool) {
        appSettings.isCustomDnsEnabled = enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setEnableCustomDns(enabled))
#elseif os(macOS)
            try? await grpcManager.setEnableCustomDns(enable: enabled)
#endif
        }
    }

    public func setTwoHop(_ enabled: Bool) {
        connectionType = enabled ? .wireguard : .mixnet5hop
        connectionConfig.enableTwoHop = enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setEnableTwoHop(enabled))
#elseif os(macOS)
            try? await grpcManager.setEnableTwoHop(enabled)
#endif
        }
    }

    public func setAdBlocking(_ enabled: Bool) {
        appSettings.isAdBlockerEnabled = enabled
        connectionConfig.enableAdBlocking = enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setEnableAdBlocking(enabled))
#elseif os(macOS)
            try? await grpcManager.setEnableAdBlocking(enabled)
#endif
        }
    }

    public func setBridges(_ enabled: Bool) {
        appSettings.isQuicEnabled = enabled
        connectionConfig.enableBridges = enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setEnableBridges(enabled))
#elseif os(macOS)
            try? await grpcManager.setEnableBridges(enabled)
#endif
        }
    }

    public func setEntryGateway(_ entry: EntryGateway) {
        entryGateway = entry
        connectionConfig.entry = entry
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setEntryPoint(entry))
#elseif os(macOS)
            try? await grpcManager.setEntryPoint(entry)
#endif
        }
    }

    public func setExitGateway(_ exit: ExitRouter) {
        exitRouter = exit
        connectionConfig.exit = exit
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setExitPoint(exit))
#elseif os(macOS)
            try? await grpcManager.setExitPoint(exit)
#endif
        }
    }

    public func applyExplicitExit(_ exit: ExitRouter) {
        setExitGateway(exit)
    }

    public func setStealthApiEnabled(_ enabled: Bool) {
        appSettings.isStealthApiEnabled = enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setFrontingModeEnabled(enabled))
#elseif os(macOS)
            try? await grpcManager.setStealthApiEnabled(enabled)
#endif
        }
    }

    public func setLanBypassEnabled(_ enabled: Bool) {
        appSettings.isLanBypassEnabled = enabled
        connectionConfig.allowLan = enabled
        Task {
#if os(macOS)
            try? await grpcManager.setAllowLan(enabled)
#endif
        }
    }

    public func setIPv6TrafficEnabled(_ enabled: Bool) {
        appSettings.isIPv6TrafficEnabled = enabled
        connectionConfig.disableIpv6 = !enabled
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setDisableIpv6(!enabled))
#elseif os(macOS)
            try? await grpcManager.setDisableIpv6(!enabled)
#endif
        }
    }

    public func setMixnetTuningConfig(_ config: MixnetTuningConfig) {
        connectionConfig.mixnetTuningConfig = config
        Task {
#if os(iOS)
            try? await sendAfterPersistingConfig(.setMixnetTrafficConfig(config))
#elseif os(macOS)
            try? await grpcManager.setMixnetTrafficConfig(config)
#endif
        }
    }

#if os(macOS)
    public func setSplitTunnelConfig(_ config: SplitTunnelConfig) {
        connectionConfig.splitTunnelConfig = config
        Task { try? await grpcManager.setSplitTunnelConfig(config) }
    }
#endif
}
