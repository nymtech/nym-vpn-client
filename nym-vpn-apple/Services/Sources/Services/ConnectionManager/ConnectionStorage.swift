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

    private var entryGatewayType: NodeType {
        connectionType == .wireguard ? .vpn : .entry
    }

    private var exitGatewayType: NodeType {
        connectionType == .wireguard ? .vpn : .exit
    }

    var connectionType: ConnectionType {
        if let typeValue = appSettings.connectionType,
           let connectionType = ConnectionType(rawValue: typeValue) {
            return connectionType
        } else {
            return ConnectionType.wireguard
        }
    }

    var entryGateway: EntryGateway {
        get {
            loadEntryGateway()
        }
        set {
            appSettings.entryGateway = newValue.toJson()
        }
    }

    var exitRouter: ExitRouter {
        get {
            loadExitRouter()
        }
        set {
            appSettings.exitRouter = newValue.toJson()
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
    /// Manipulates gateway if last parameter does not exist anymore.
    /// Example: Checks if country exists, if not returns Switzerland, if Switzerland does not exist - first country.
    /// Example: If mixnet server does not support vpn, will return country.
    /// - Returns: EntryGateway
    func loadEntryGateway() -> EntryGateway {
        let jsonString = appSettings.entryGateway ?? ""
        guard let gateway = EntryGateway.from(jsonString: jsonString)
        else {
            // Fallback to Switzerland or first country
            return .country(fallbackCountry(nodeType: entryGatewayType).code)
        }

        switch gateway {
        case let .country(code):
            let existingCountry = existingCountry(with: code, nodeType: entryGatewayType)
            return .country(existingCountry.code)
        case let .lowLatencyCountry(code):
            let country = existingCountry(with: code, nodeType: entryGatewayType)
            return .lowLatencyCountry(country.code)
        case let .gateway(identifier):
            if let existingGateway = existingGateway(with: identifier, nodeType: entryGatewayType) {
                return .gateway(existingGateway.id)
            } else {
                let existingCountry = existingCountry(
                    with: gatewayManager.country(with: identifier, nodeType: entryGatewayType)?.code
                    ?? fallbackCountry(nodeType: .entry).code,
                    nodeType: entryGatewayType
                )
                return .country(existingCountry.code)
            }
        case let .region(countryCode: code, region: region):
            return .region(countryCode: code, region: region)
        case let .city(city):
            return .city(city)
        case .random:
            return .random
        }
    }

    /// Manipulates router if last parameter does not exist anymore.
    /// Example: Checks if country exists, if not returns Switzerland, if Switzerland does not exist - first country.
    /// - Returns: ExitRouter
    func loadExitRouter() -> ExitRouter {
        let jsonString = appSettings.exitRouter ?? ""
        guard let router = ExitRouter.from(jsonString: jsonString)
        else {
            return .country(fallbackCountry(nodeType: exitGatewayType).code)
        }

        switch router {
        case let .country(code):
            let existingCountry = existingCountry(with: code, nodeType: exitGatewayType)
            return .country(existingCountry.code)
        case let .gateway(identifier):
            if let existingGateway = existingGateway(with: identifier, nodeType: exitGatewayType) {
                return .gateway(existingGateway.id)
            } else {
                let existingCountry = existingCountry(
                    with: gatewayManager.country(with: identifier, nodeType: exitGatewayType)?.code
                    ?? fallbackCountry(nodeType: .exit).code,
                    nodeType: exitGatewayType
                )
                return .country(existingCountry.code)
            }
        case let .address(address):
            return .address(address)
        case let .region(countryCode: code, region: region):
            return .region(countryCode: code, region: region)
        case .random:
            return .random
        }
    }
}

// MARK: - Countries -
private extension ConnectionStorage {
    /// Checks if selected gateway country exists. If not - returns first country from the country list, if no countries present - returns Switzerland
    /// - Parameter countryCode: String
    /// - Parameter isEntryHop: Bool. Determines from which country array(entry/exit) to return the country from
    /// - Returns: String with countryCode
    func existingCountry(with countryCode: String, nodeType: NodeType) -> NymCountry {
        if let country = gatewayManager.country(with: countryCode, gatewayType: nodeType) {
            return country
        } else {
            return fallbackCountry(nodeType: nodeType)
        }
    }

    func fallbackCountry(nodeType: NodeType) -> NymCountry {
        let fallbackCountry = NymCountry(name: "Switzerland", code: "CH", regions: [])
        switch nodeType {
        case .entry:

            if gatewayManager.entryCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = gatewayManager.entryCountries.first {
                return country
            }
        case .exit:
            if gatewayManager.exitCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = gatewayManager.exitCountries.first {
                return country
            }
        case .vpn:
            if gatewayManager.vpnCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = gatewayManager.vpnCountries.first {
                return country
            }
        }
        return fallbackCountry
    }
}

// MARK: - Gateways -
private extension ConnectionStorage {
    func existingGateway(with gatewayId: String, nodeType: NodeType) -> GatewayNode? {
        switch nodeType {
        case .entry:
            gatewayManager.entry.first { $0.id == gatewayId }
        case .exit:
            gatewayManager.exit.first { $0.id == gatewayId }
        case .vpn:
            gatewayManager.vpn.first { $0.id == gatewayId }
        }
    }
}
