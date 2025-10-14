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
    private let resolver = GatewaySelectionResolver()

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

    var resolvedEntryGateway: EntryGateway {
        get async {
            let json = appSettings.entryGateway ?? ""
            let snapshotEntry = gatewayManager.entry
            let snapshotExit  = gatewayManager.exit
            let snapshotVPN   = gatewayManager.vpn
            let entryCountries = gatewayManager.entryCountries
            let exitCountries  = gatewayManager.exitCountries
            let vpnCountries   = gatewayManager.vpnCountries

            return await resolver.resolveEntryGateway(
                jsonString: json,
                connectionType: connectionType,
                entry: snapshotEntry,
                exit: snapshotExit,
                vpn: snapshotVPN,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
        }
    }

    var resolvedExitRouter: ExitRouter {
        get async {
            let json = appSettings.exitRouter ?? ""
            let snapshotEntry = gatewayManager.entry
            let snapshotExit  = gatewayManager.exit
            let snapshotVPN   = gatewayManager.vpn
            let entryCountries = gatewayManager.entryCountries
            let exitCountries  = gatewayManager.exitCountries
            let vpnCountries   = gatewayManager.vpnCountries

            return await resolver.resolveExitRouter(
                jsonString: json,
                connectionType: connectionType,
                entry: snapshotEntry,
                exit: snapshotExit,
                vpn: snapshotVPN,
                entryCountries: entryCountries,
                exitCountries: exitCountries,
                vpnCountries: vpnCountries
            )
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
