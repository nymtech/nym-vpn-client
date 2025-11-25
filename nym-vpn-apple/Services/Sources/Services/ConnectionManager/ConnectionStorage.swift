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
        if let storedType = appSettings.connectionType,
           let type = ConnectionType(rawValue: storedType) {
            type
        } else {
            .wireguard
        }
    }

    var entryGateway: EntryGateway {
        get { EntryGateway.from(jsonString: appSettings.entryGateway ?? "") ?? .country("CH") }
        set { appSettings.entryGateway = newValue.toJson() }
    }

    var exitRouter: ExitRouter {
        get { ExitRouter.from(jsonString: appSettings.exitRouter ?? "") ?? .country("CH") }
        set { appSettings.exitRouter = newValue.toJson() }
    }

    var connectionConfig: ConnectionConfig {
        get {
            guard let storedConfig = appSettings.connectionConfig,
                  let decodedConfig = ConnectionConfig.from(jsonString: storedConfig)
            else {
                return generateInitialConfig()
            }
            return decodedConfig
        }
        set {
            appSettings.connectionConfig = newValue.toJson()
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
    }
}

private extension ConnectionStorage {
    func generateInitialConfig() -> ConnectionConfig {
        ConnectionConfig(
            entry: entryGateway,
            exit: exitRouter,
            allowLan: false,
            disableIpv6: !appSettings.isIPv6TrafficEnabled,
            enableTwoHop: connectionType == .wireguard,
            enableBridges: appSettings.isQuicEnabled,
            netstack: false,
            disablePoissonRate: false,
            disableBackgroundCoverTraffic: false,
            residentialExit: false
        )
    }
}
