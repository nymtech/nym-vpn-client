import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import CountriesManagerTypes
import GatewayManager
import UIComponents

@MainActor public class GatewaysViewModel: ObservableObject {
    private let connectionManager: ConnectionManager
    private let countriesManager: CountriesManager
    private let gatewayManager: GatewayManager

    let type: HopType
    let minimumSearchSymbols = 2

    @Binding var path: NavigationPath
    @Published var isGeolocationModalDisplayed = false
    @Published var isServerInfoModalDisplayed = false
    @Published var serverInfoModalServer: GatewayNode?
    @Published var gateways = [GatewayNode]()
    @Published var countries = [Country]()
    @Published var scrollToServer: GatewayNode?
    @Published var foundCountries = [Country]()
    @Published var foundGateways = [GatewayNode]()
    @Published var searchText: String = "" {
        didSet {
            searchCountriesGateways()
        }
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        connectionManager: ConnectionManager = .shared,
        countriesManager: CountriesManager = .shared,
        gatewayManager: GatewayManager = .shared
    ) {
        _path = path
        self.type = type
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager
        self.gatewayManager = gatewayManager

        setup()
    }
}

@MainActor extension GatewaysViewModel {
    func gatewaysInCountry(with countryCode: String) -> [GatewayNode] {
        gateways.filter { $0.countryCode == countryCode }
    }
}

// MARK: - Setup -
@MainActor private extension GatewaysViewModel {
    func setup() {
        updateGateways()
    }
}

// MARK: - Navigation -
@MainActor extension GatewaysViewModel {
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
@MainActor private extension GatewaysViewModel {
    func updateGateways() {
        switch connectionManager.connectionType {
        case .mixnet5hop:
            switch type {
            case .entry:
                gateways = gatewayManager.entry
            case .exit:
                gateways = gatewayManager.exit
            }
        case .wireguard:
            gateways = gatewayManager.vpn
        }
        let countryCodes = Array(Set(gateways.map { $0.countryCode }))
        countries = countryCodes.compactMap {
            countriesManager.country(with: $0)
        }
        .sorted(by: { $0.name < $1.name })
    }

    func searchCountriesGateways() {
        guard searchText.count >= minimumSearchSymbols else { return }
        foundCountries = countries.filter {
            $0.name.lowercased().contains(searchText.lowercased())
            || $0.code.lowercased().contains(searchText.lowercased())
        }
        foundGateways = gateways.filter {
            $0.moniker?.lowercased().contains(searchText.lowercased()) ?? false
            || $0.id.lowercased().contains(searchText.lowercased())
        }
    }
}
