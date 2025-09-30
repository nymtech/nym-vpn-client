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

public final class GatewayManager: ObservableObject {
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

    public static let shared = GatewayManager()

    @Published public var entry: [GatewayNode]
    @Published public var exit: [GatewayNode]
    @Published public var vpn: [GatewayNode]
    @Published public var entryCountries: [Country]
    @Published public var exitCountries: [Country]
    @Published public var vpnCountries: [Country]
    @Published public var lastError: Error?

    lazy var iso8601Flexible: ISO8601DateFormatter = {
        let formatter = ISO8601DateFormatter()
        formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return formatter
    }()

#if os(iOS)
    public init(appSettings: AppSettings = .shared, configurationManager: ConfigurationManager = .shared) {
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
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        grpcManager: GRPCManager = .shared
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
    public func country(with code: String, gatewayType: NodeType) -> Country? {
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
            return country(with: gateway.location?.twoLetterIsoCountryCode)
        } else {
            return nil
        }
    }

    /// Localized country
    /// - Parameter countryCode: String
    /// - Returns: Country
    public func country(with countryCode: String?) -> Country? {
        guard let countryCode,
              !countryCode.isEmpty,
              let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            return nil
        }
        return Country(name: countryName, code: countryCode)
    }
    
    /// Country from gateway id for node type
    /// - Parameters:
    ///   - gatewayId: String
    ///   - nodeType: NodeType
    /// - Returns: Country?
    public func country(with gatewayId: String?, nodeType: NodeType) -> Country? {
        guard let gatewayId else { return nil }
        switch nodeType {
        case .entry:
            let code = entry.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return country(with: code)
        case .exit:
            let code = exit.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return country(with: code)
        case .vpn:
            let code = vpn.first { $0.id == gatewayId }?.location?.twoLetterIsoCountryCode
            return country(with: code)
        }
    }

    public func countryCode(with gateway: EntryGateway) -> String? {
        switch gateway {
        case let .country(code):
            return code
        case let .region(region):
            return nil
        case .city(let string):
            return nil
        case let .lowLatencyCountry(code):
            return code
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .entry)?.code ?? country(with: identifier, nodeType: .vpn)?.code
        case .random:
            return nil
        }
    }

    public func countryCode(with router: ExitRouter) -> String? {
        switch router {
        case let .address(string):
            return nil
        case let .country(code):
            return code
        case let .gateway(identifier):
            return country(with: identifier, nodeType: .exit)?.code ?? country(with: identifier, nodeType: .vpn)?.code
        case .region:
            return nil
        case .random:
            return nil
        }
    }
}

private extension GatewayManager {
    func setupAutoUpdates() {
        timer = Timer.scheduledTimer(
            timeInterval: 600,
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
                try? await Task.sleep(for: .seconds(7))
                await self?.fetchGateways()
            }
        }
    }

    func updateCountriesFromGateways() {
        entryCountries = countries(from: entry)
        exitCountries = countries(from: exit)
        vpnCountries = countries(from: vpn)
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
    func countries(from nodes: [GatewayNode]) -> [Country] {
        let codes = nodes.compactMap { $0.location?.twoLetterIsoCountryCode }
        let countries = codes.compactMap { country(with: $0) }
            .sorted(by: { $0.name < $1.name })
        return countries
    }
}
