import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManagerTypes
import GatewayManager
import UIComponents

@MainActor public class GatewaysViewModel: ObservableObject {
    private let gatewayManager: GatewayManager

    let type: HopType
    let minimumSearchSymbols = 2

    @ObservedObject var connectionManager: ConnectionManager
    @Binding var path: NavigationPath
    @Published var isGeolocationModalDisplayed = false
    @Published var gateways = [GatewayNode]()
    @Published var countries = [Country]()
    @Published var foundCountries = [Country]()
    @Published var foundGateways = [GatewayNode]()
    @Published var scrollToModel: GatewayScrollToModel
    @Published var searchText: String = "" {
        didSet {
            searchCountriesGateways()
        }
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        connectionManager: ConnectionManager,
        gatewayManager: GatewayManager
    ) {
        _path = path
        self.type = type
        self.connectionManager = connectionManager
        self.gatewayManager = gatewayManager

        switch type {
        case .entry:
            scrollToModel = .init(entryGateaway: connectionManager.entryGateway)
        case .exit:
            scrollToModel = .init(exitRouter: connectionManager.exitRouter)
        }
        setup()
    }
}

@MainActor extension GatewaysViewModel {
    func gatewaysInCountry(with countryCode: String) -> [GatewayNode] {
        gateways.filter { $0.location?.twoLetterIsoCountryCode == countryCode }
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
        countries = Array(Set(gateways.map { $0.location?.twoLetterIsoCountryCode }))
            .compactMap { gatewayManager.localizedCountry(with: $0) }
            .sorted {
                $0.name.compare(
                    $1.name,
                    options: [.caseInsensitive, .diacriticInsensitive, .widthInsensitive],
                    range: nil,
                    locale: Locale.current
                ) == .orderedAscending
            }
    }

    func searchCountriesGateways() {
        guard searchText.count >= minimumSearchSymbols
        else {
            foundCountries = [Country]()
            foundGateways = [GatewayNode]()
            return
        }
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
