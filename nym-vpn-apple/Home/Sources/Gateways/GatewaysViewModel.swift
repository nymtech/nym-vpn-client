import Combine
import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManagerTypes
import FeatureFlagsManager
import GatewayManager
import UIComponents

@MainActor public class GatewaysViewModel: ObservableObject {
    private var cancellables = Set<AnyCancellable>()

    let gatewayManager: GatewayManager
    let type: HopType
    let minimumSearchSymbols = 2
    
    var lastError: Error?

    @ObservedObject var appSettings: AppSettings
    @ObservedObject var connectionManager: ConnectionManager
    @ObservedObject var featureFlagsManager: FeatureFlagsManager
    @Binding var path: NavigationPath
    @Published var isRefreshing = false
    @Published var isServerListRefreshFailedOverlayDisplayed = false
    @Published var isGeolocationModalDisplayed = false
    @Published var gateways = [GatewayNode]()
    @Published var countries = [NymCountry]()
    @Published var foundCountries = [NymCountry]()
    @Published var foundRegions = [(country: NymCountry, region: String)]()
    @Published var foundGateways = [GatewayNode]()
    @Published var scrollToModel: GatewayScrollToModel
    @Published var shouldScroll = false
    @Published var searchText: String = "" {
        didSet {
            searchCountriesGateways()
        }
    }

    var shouldShowQuic: Bool {
        type == .entry
        && connectionManager.connectionType == .wireguard
        && appSettings.isQuicEnabled
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        gatewayManager: GatewayManager,
        featureFlagsManager: FeatureFlagsManager
    ) {
        _path = path
        self.type = type
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.gatewayManager = gatewayManager
        self.featureFlagsManager = featureFlagsManager

        switch type {
        case .entry:
            scrollToModel = .init(entryGateaway: connectionManager.entryGateway)
        case .exit:
            scrollToModel = .init(exitRouter: connectionManager.exitRouter)
        }
        setup()
    }
}

extension GatewaysViewModel {
    func gatewaysInCountry(with countryCode: String) -> [GatewayNode] {
        gateways.filter {
            $0.location?.twoLetterIsoCountryCode.caseInsensitiveCompare(countryCode) == .orderedSame
        }
        .sorted {
            ($0.performance?.score.rawValue ?? .max) < ($1.performance?.score.rawValue ?? .max)
        }
    }

    @MainActor func refreshServersList() async {
        isRefreshing = true
        await gatewayManager.refresh()
        updateGateways()
        isRefreshing = false
    }
}

// MARK: - Setup -
private extension GatewaysViewModel {
    func setup() {
        updateGateways()
        setupQuicToggleObserver()
    }

    func setupQuicToggleObserver() {
        appSettings.$isQuicEnabledPublisher
            .removeDuplicates()
            .sink { [weak self] _ in
                self?.updateGateways()
            }
            .store(in: &cancellables)
    }
    
    func setupGatewayManagerObserver() {
        gatewayManager.$lastError.sink { [weak self] error in
            self?.lastError = error
        }
        .store(in: &cancellables)
    }
}

// MARK: - Navigation -
extension GatewaysViewModel {
    func navigateHome() {
        path = .init()
    }

    func displayInfoTooltip() {
        withAnimation {
            isGeolocationModalDisplayed.toggle()
        }
    }
}

// MARK: - Gateways -
private extension GatewaysViewModel {
    func updateGateways() {
        switch connectionManager.connectionType {
        case .mixnet5hop:
            switch type {
            case .entry:
                gateways = gatewayManager.entry
                countries = gatewayManager.entryCountries
            case .exit:
                gateways = gatewayManager.exit
                countries = gatewayManager.exitCountries
            }
        case .wireguard:
            if shouldShowQuic {
                gateways = gatewayManager.vpn.filter { $0.isQuicAvailable }
            } else {
                gateways = gatewayManager.vpn
            }
            countries = gatewayManager.vpnCountries
        }
        shouldScroll = true
    }

    func searchCountriesGateways() {
        guard searchText.count >= minimumSearchSymbols
        else {
            foundCountries = [NymCountry]()
            foundGateways = [GatewayNode]()
            return
        }
        let newCountries = countries.filter {
            $0.name.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased())
            || $0.code.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased())
        }

        // TODO: city update to use new country with found regions or cities
        var seen = Set<String>()
        let newCountryRegionPairs: [(country: NymCountry, region: String)] = gateways
            .compactMap { gateway -> (NymCountry, String)? in
                guard let location = gateway.location,
                      self.gatewayManager.countriesSupportingRegions.contains(
                        where: {
                            $0.caseInsensitiveCompare(location.twoLetterIsoCountryCode) == .orderedSame
                        }
                      ),
                      !location.region.isEmpty,
                      location.region.range(
                        of: self.searchText, options: [.caseInsensitive, .diacriticInsensitive]
                      ) != nil,
                      let country = self.gatewayManager.localizedCountry(with: location.twoLetterIsoCountryCode),
                      seen.insert(location.region).inserted
                else {
                    return nil
                }
                return (country, location.region)
            }

        let newGateways = gateways.filter {
            $0.name?.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased()) ?? false
            || $0.id.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased())
        }
        foundCountries = newCountries
        foundRegions = newCountryRegionPairs
        foundGateways = newGateways
    }
}

extension GatewaysViewModel: Equatable, Hashable {
    nonisolated public static func == (lhs: GatewaysViewModel, rhs: GatewaysViewModel) -> Bool {
        lhs.type == rhs.type
    }

    nonisolated public func hash(into hasher: inout Hasher) {
        hasher.combine(type)
    }
}
