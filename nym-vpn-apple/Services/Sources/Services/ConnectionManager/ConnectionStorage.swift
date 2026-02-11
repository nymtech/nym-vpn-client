import AppSettings
import ConfigurationManager
import ConnectionTypes
import CountriesManagerTypes
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
        willSet {
            previousConnectionConfig = connectionConfig
        }
        didSet {
            appSettings.connectionConfig = connectionConfig.toJson()
        }
    }

    var previousConnectionConfig: ConnectionConfig?

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

private extension ConnectionStorage {
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
            entry: .country("CH"),
            exit: .country("CH"),
            dns: nil,
            allowLan: false,
            disableIpv6: false,
            enableTwoHop: true,
            enableBridges: false,
            enableLewes: false,
            netstack: false,
            residentialExit: false,
            mixnetTuningConfig: MixnetTuningConfig(
                backgroundTraffic: BackgroundTraffic(actualValue: nil),
                continuousTraffic: ContinuousTraffic(actualValue: nil),
                dissablePoissonRate: false
            )
        )
    }
}
