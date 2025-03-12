import AppSettings
import ConfigurationManager
import ConnectionTypes
import CountriesManager
import CountriesManagerTypes
import GatewayManager

public final class ConnectionStorage {
    public static let shared = ConnectionStorage()

    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let countriesManager: CountriesManager
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
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        countriesManager: CountriesManager = .shared,
        gatewayManager: GatewayManager = .shared
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.countriesManager = countriesManager
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
            return .country(fallbackCountry(nodeType: entryGatewayType))
        }

        switch gateway {
        case let .country(country):
            let existingCountry = existingCountry(with: country.code, nodeType: entryGatewayType)
            return .country(existingCountry)
        case let .lowLatencyCountry(country):
            let country = existingCountry(with: country.code, nodeType: entryGatewayType)
            return .lowLatencyCountry(country)
        case let .gateway(gateway):
            if let existingGateway = existingGateway(with: gateway, nodeType: entryGatewayType) {
                return .gateway(existingGateway)
            } else {
                let existingCountry = existingCountry(with: gateway.countryCode, nodeType: entryGatewayType)
                return .country(existingCountry)
            }
        case .randomLowLatency:
            return .randomLowLatency
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
            return .country(fallbackCountry(nodeType: exitGatewayType))
        }

        switch router {
        case let .country(country):
            let existingCountry = existingCountry(with: country.code, nodeType: exitGatewayType)
            return .country(existingCountry)
        case let .gateway(gateway):
            if let existingGateway = existingGateway(with: gateway, nodeType: exitGatewayType) {
                return .gateway(existingGateway)
            } else {
                let existingCountry = existingCountry(with: gateway.countryCode, nodeType: exitGatewayType)
                return .country(existingCountry)
            }
        }
    }
}

// MARK: - Countries -
private extension ConnectionStorage {
    /// Checks if selected gateway country exists. If not - returns first country from the country list, if no countries present - returns Switzerland
    /// - Parameter countryCode: String
    /// - Parameter isEntryHop: Bool. Determines from which country array(entry/exit) to return the country from
    /// - Returns: String with countryCode
    func existingCountry(with countryCode: String, nodeType: NodeType) -> Country {
        if let country = countriesManager.country(with: countryCode, gatewayType: nodeType) {
            return country
        } else {
            return fallbackCountry(nodeType: nodeType)
        }
    }

    func fallbackCountry(nodeType: NodeType) -> Country {
        let fallbackCountry = Country(name: "Switzerland", code: "CH")
        switch nodeType {
        case .entry:
            if countriesManager.entryCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = countriesManager.entryCountries.first {
                return country
            }
        case .exit:
            if countriesManager.exitCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = countriesManager.exitCountries.first {
                return country
            }
        case .vpn:
            if countriesManager.vpnCountries.contains(where: { $0.code == "CH" }) {
                return fallbackCountry
            } else if let country = countriesManager.vpnCountries.first {
                return country
            }
        }
        return fallbackCountry
    }
}

// MARK: - Gateways -
private extension ConnectionStorage {
    func existingGateway(with gateway: GatewayNode, nodeType: NodeType) -> GatewayNode? {
        switch nodeType {
        case .entry:
            gatewayManager.entry.first { $0.id == gateway.id }
        case .exit:
            gatewayManager.exit.first { $0.id == gateway.id }
        case .vpn:
            gatewayManager.vpn.first { $0.id == gateway.id }
        }
    }
}
