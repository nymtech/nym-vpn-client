import AppSettings
import ConfigurationManager
import ConnectionTypes
import GatewayManager

@MainActor public final class ConnectionStorage {
    public static let shared = ConnectionStorage(
        appSettings: .shared,
        configurationManager: .shared,
        gatewayManager: .shared
    )

    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let gatewayManager: GatewayManager

    private var entryGatewayType: NodeType { connectionType == .wireguard ? .vpn : .entry }
    private var exitGatewayType: NodeType { connectionType == .wireguard ? .vpn : .exit }

    var connectionType: ConnectionType {
        connectionConfig.enableTwoHop ? .wireguard : .mixnet5hop
    }

    var entryGateway: EntryGateway {
        connectionConfig.entry
    }
    var exitRouter: ExitRouter {
        connectionConfig.exit
    }

    var connectionConfig: ConnectionConfig {
        didSet {
            appSettings.connectionConfig = connectionConfig.toJson()
        }
    }

    public init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        gatewayManager: GatewayManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.gatewayManager = gatewayManager
        self.connectionConfig = Self.decodeStoredConfig(appSettings: appSettings)
    }
}

extension ConnectionStorage {
    /// True when the in-memory ConnectionConfig is byte-identical to the
    /// freshly generated initial config — i.e. nothing has been changed yet.
    /// Used by the macOS first-launch bootstrap to detect a fresh install.
    public var isUsingInitialConfig: Bool {
        connectionConfig == Self.generateInitialConfig()
    }

#if SANTA
    func resetGatewaySelectionsForEnvironmentChange() {
        connectionConfig.entry = .auto
        connectionConfig.exit = .auto
    }

    func registerForEnvironmentChanges(onReset: @escaping () -> Void) {
        configurationManager.addEnvironmentDidChangeObserver(onReset)
    }
#endif
}

extension ConnectionStorage {
    static func decodeStoredConfig(appSettings: AppSettings) -> ConnectionConfig {
        guard let storedConfig = appSettings.connectionConfig,
              let decodedConfig = ConnectionConfig.from(jsonString: storedConfig)
        else {
            return generateInitialConfig()
        }
        return decodedConfig
    }

    static func generateInitialConfig() -> ConnectionConfig {
        ConnectionConfig(
            entry: .auto,
            exit: .auto,
            dns: nil,
            allowLan: false,
            disableIpv6: false,
            enableTwoHop: true,
            enableBridges: false,
            enableAdBlocking: false,
            netstack: false,
            residentialExit: false,
            mixnetTuningConfig: MixnetTuningConfig(
                backgroundTraffic: .ms200,
                continuousTraffic: .ms20,
                dissablePoissonRate: false
            ),
            splitTunnelConfig: SplitTunnelConfig(),
            gatewaySelectionAlgorithmConfig: NymGatewaySelectionAlgorithmConfig(
                enableGeoLocation: true
            )
        )
    }
}
