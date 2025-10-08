import Combine
import Foundation
import AppSettings
import ConfigurationManager
import ConnectionTypes
import CountriesManagerTypes
import Logging
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif

@MainActor public final class GatewayManager: ObservableObject {
    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
#if os(macOS)
    let grpcManager: GRPCManager
#endif
    let logger = Logger(label: "GatewayManager")

    var isLoading = false
    var timer: Timer?
    var gatewayStore = GatewayNodeStore()
    var cancellables = Set<AnyCancellable>()

#if os(iOS)
    public static let shared = GatewayManager(appSettings: .shared, configurationManager: .shared)
#elseif os(macOS)
    public static let shared = GatewayManager(
        appSettings: .shared,
        configurationManager: .shared,
        grpcManager: .shared
    )
#endif

    @Published public var entry: [GatewayNode]
    @Published public var exit: [GatewayNode]
    @Published public var vpn: [GatewayNode]
    @Published public var entryCountries: [NymCountry]
    @Published public var exitCountries: [NymCountry]
    @Published public var vpnCountries: [NymCountry]
    @Published public var lastError: Error?

    lazy var iso8601Flexible: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

#if os(iOS)
    public init(appSettings: AppSettings, configurationManager: ConfigurationManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.entry = []
        self.exit = []
        self.vpn = []
        self.entryCountries = []
        self.exitCountries = []
        self.vpnCountries = []
        loadGatewayStore()
        loadPrebundledServersIfNecessary()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        self.entry = []
        self.exit = []
        self.vpn = []
        self.entryCountries = []
        self.exitCountries = []
        self.vpnCountries = []
        loadGatewayStore()
        loadPrebundledServersIfNecessary()
        setupDaemonObserver()
    }
#endif

    /// Run from NymVpnApp
    public func setup() {
        updateGateways()
        setupAutoUpdates()
        configureEnvironmentChange()
    }

    public func moniker(with gatewayId: String?) -> String? {
        entry.first(where: { $0.id == gatewayId })?.moniker
        ?? exit.first(where: { $0.id == gatewayId })?.moniker
        ?? vpn.first(where: { $0.id == gatewayId })?.moniker
    }

    /// Returns country from isoCode if it exists in the gateways
    /// - Parameters:
    ///   - code: countryCode
    ///   - gatewayType: gateway type
    /// - Returns: Countrry if any of the gateways are located in it or nil.
    public func country(with code: String, gatewayType: NodeType) -> NymCountry? {
        let gateway: GatewayNode?
        switch gatewayType {
        case .entry:
            gateway = entry.first(where: { $0.location?.twoLetterIsoCountryCode == code })
        case .exit:
            gateway = exit.first(where: { $0.location?.twoLetterIsoCountryCode == code })
        case .vpn:
            gateway = vpn.first(where: { $0.location?.twoLetterIsoCountryCode == code })
        }
        if let gateway {
            return localizedCountry(with: gateway.location?.twoLetterIsoCountryCode)
        } else {
            return nil
        }
    }

    /// Localized country
    /// - Parameter countryCode: String
    /// - Returns: Country
    public func localizedCountry(with countryCode: String?) -> NymCountry? {
        guard let countryCode,
              !countryCode.isEmpty,
              let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            return nil
        }
        return NymCountry(name: countryName, code: countryCode, regions: [])
    }

    /// Country from gateway id for node type
    /// - Parameters:
    ///   - gatewayId: String
    ///   - nodeType: NodeType
    /// - Returns: Country?
    public func country(with gatewayId: String?, nodeType: NodeType) -> NymCountry? {
        guard let gatewayId else { return nil }
        switch nodeType {
        case .entry:
            let code = entry.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return localizedCountry(with: code)
        case .exit:
            let code = exit.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return localizedCountry(with: code)
        case .vpn:
            let code = vpn.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return localizedCountry(with: code)
        }
    }

    public func countryCode(with gateway: EntryGateway) -> String? {
        switch gateway {
        case let .country(code), let .lowLatencyCountry(code):
            return code
        case let .region(countryCode: code, region: _):
            return localizedCountry(with: code)?.code
        case .city:
            return nil
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .entry)?.code ?? country(with: identifier, nodeType: .vpn)?.code
        case .random:
            return nil
        }
    }

    public func countryCode(with router: ExitRouter) -> String? {
        switch router {
        case .address:
            return nil
        case let .country(code):
            return code
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .exit)?.code ?? country(with: identifier, nodeType: .vpn)?.code
        case let .region(countryCode: code, region: _):
            return localizedCountry(with: code)?.code
        case .random:
            return nil
        }
    }

    public func userFriendlyTitle(with gateway: EntryGateway) -> String? {
        switch gateway {
        case let .country(code), let .lowLatencyCountry(code):
            return localizedCountry(with: code)?.name
        case let .region(countryCode: code, region: region):
            if let country = localizedCountry(with: code) {
                return "\(country.name), \(region)"
            } else {
                return region
            }
        case .city:
            return nil
        case let .gateway(identifier):
            return moniker(with: identifier) ?? identifier
        case .random:
            return nil
        }
    }

    public func userFriendlyTitle(with router: ExitRouter) -> String? {
        switch router {
        case .address:
            return nil
        case let .country(code):
            return localizedCountry(with: code)?.name
        case let .gateway(identifier):
            return moniker(with: identifier) ?? identifier
        case let .region(countryCode: code, region: region):
            if let country = localizedCountry(with: code) {
                return "\(country.name), \(region)"
            } else {
                return region
            }
        case .random:
            return nil
        }
    }
}

extension GatewayManager {
    func updateCountriesFromGateways() {
        entryCountries = countries(from: entry)
        exitCountries = countries(from: exit)
        vpnCountries = countries(from: vpn)
    }
}

private extension GatewayManager {
    func setupAutoUpdates() {
        timer = Timer.scheduledTimer(
            timeInterval: 300,
            target: self,
            selector: #selector(updateGateways),
            userInfo: nil,
            repeats: true
        )
    }

    @objc func updateGateways() {
        guard !isLoading, needsReload()
        else {
            if entry.isEmpty
                || exit.isEmpty
                || vpn.isEmpty {
                loadGatewaysFromStore()
            }
            return
        }
        isLoading = true

        Task { [weak self] in
            await self?.fetchGateways()
        }
    }
    func needsReload() -> Bool {
        guard let lastFetchDate = gatewayStore.lastFetchDate else { return true }
        return isLongerThan10Minutes(date: lastFetchDate)
    }

    func isLongerThan10Minutes(date: Date) -> Bool {
        Date().timeIntervalSince(date) > 600 ? true : false
    }

    func loadGatewaysFromStore() {
        Task { @MainActor in
            exit = gatewayStore.exit
            entry = gatewayStore.entry
            vpn = gatewayStore.vpn
        }
    }

    func configureEnvironmentChange() {
        configurationManager.environmentDidChange = { [weak self] in
            self?.gatewayStore.lastFetchDate = nil
            Task {
                try? await Task.sleep(for: .seconds(3))
                await self?.fetchGateways()
            }
        }
    }
}

extension GatewayManager {
    func updateError(with error: Error) {
        Task { @MainActor in
            lastError = error
        }
    }
}

private extension GatewayManager {
    func countries(from nodes: [GatewayNode]) -> [NymCountry] {
        var regionsByCode: [String: Set<String>] = [:]
        nodes.compactMap(\.location).forEach { location in
            let code = location.twoLetterIsoCountryCode.uppercased()
            let region = location.region.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !region.isEmpty else { return }
            regionsByCode[code, default: []].insert(region)
        }

        var result: [NymCountry] = []
        result.reserveCapacity(regionsByCode.count)

        regionsByCode.forEach { code, regionsSet in
            guard var country = localizedCountry(with: code) else { return }
            country.regions = regionsSet.sorted {
                $0.localizedCaseInsensitiveCompare($1) == .orderedAscending
            }
            result.append(country)
        }

        result.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        return result
    }
}
