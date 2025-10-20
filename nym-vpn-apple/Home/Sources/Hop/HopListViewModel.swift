import SwiftUI
import AppSettings
import ConfigurationManager
import ConnectionManager
import CountriesManagerTypes
import GatewayManager
import UIComponents

@MainActor public class HopListViewModel: ObservableObject {
    let type: HopType

    public let noResultsText = "search.noResults".localizedString

    var appSettings: AppSettings
    var configurationManager: ConfigurationManager
    var connectionManager: ConnectionManager
    let gatewayManager: GatewayManager
    @Binding var path: NavigationPath

    @Published var isGeolocationModalDisplayed = false
    @Published var quickestCountry: NymCountry?
    @Published var countries: [NymCountry]?
    @Published var searchText: String = "" {
        didSet {
            updateCountries()
        }
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        connectionManager: ConnectionManager,
        gatewayManager: GatewayManager
    ) {
        _path = path
        self.type = type
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.gatewayManager = gatewayManager
        setup()
    }

    func connectionSelect(with country: NymCountry) {
        switch type {
        case .entry:
            connectionManager.entryGateway = .country(country.code)
        case .exit:
            connectionManager.exitRouter = .country(country.code)
        }
        navigateHome()
    }

    func connectionSelect(with gateway: GatewayNode) {
        switch type {
        case .entry:
            connectionManager.entryGateway = .gateway(gateway.id)
        case .exit:
            connectionManager.exitRouter = .gateway(gateway.id)
        }
        navigateHome()
    }

    func quickestConnectionSelect(with country: NymCountry) {
        switch type {
        case .entry:
            connectionManager.entryGateway = .lowLatencyCountry(country.code)
        case .exit:
            break
        }
        navigateHome()
    }

    func isCountrySelected(countryCode: String) -> Bool {
        switch type {
        case .entry:
            return connectionManager.entryGateway.countryCode == countryCode
        case .exit:
            return connectionManager.exitRouter.countryCode == countryCode
        }
    }

    func displayInfoTooltip() {
        isGeolocationModalDisplayed.toggle()
    }
}

// MARK: - Navigation -
extension HopListViewModel {
    func navigateHome() {
        path = .init()
    }
}

// MARK: - Setup -
private extension HopListViewModel {
    func setup() {
        updateCountries()
    }
}

// MARK: - Countries -
private extension HopListViewModel {
    func updateCountries() {
        Task { [weak self] in
            guard let self else { return }
            let newCountries: [NymCountry]?
            switch connectionManager.connectionType {
            case .mixnet5hop:
                newCountries = countriesMixnet()
            case .wireguard:
                newCountries = countriesWireGuard()
            }
            await MainActor.run {
                self.countries = newCountries
            }
        }
    }

    func countriesMixnet() -> [NymCountry] {
        switch type {
        case .entry:
            return !searchText.isEmpty ? gatewayManager.entryCountries.filter {
                $0.name.lowercased().contains(searchText.lowercased()) ||
                $0.code.lowercased().contains(searchText.lowercased())
            } : gatewayManager.entryCountries
        case .exit:
            return !searchText.isEmpty ? gatewayManager.exitCountries.filter {
                $0.name.lowercased().contains(searchText.lowercased()) ||
                $0.code.lowercased().contains(searchText.lowercased())
            } : gatewayManager.exitCountries
        }
    }

    func countriesWireGuard() -> [NymCountry] {
        !searchText.isEmpty ? gatewayManager.vpnCountries.filter {
            $0.name.lowercased().contains(searchText.lowercased()) ||
            $0.code.lowercased().contains(searchText.lowercased())
        } : gatewayManager.vpnCountries
    }
}
