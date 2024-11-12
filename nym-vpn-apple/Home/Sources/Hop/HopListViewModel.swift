import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import CountriesManagerTypes
import UIComponents

public class HopListViewModel: ObservableObject {
    let type: HopType

    public let noResultsText = "search.noResults".localizedString

    var appSettings: AppSettings
    var connectionManager: ConnectionManager
    var countriesManager: CountriesManager
    @Binding var path: NavigationPath

    @Published var isGeolocationModalDisplayed = false
    @Published var quickestCountry: Country?
    @Published var countries: [Country]?
    @Published var searchText: String = "" {
        didSet {
            updateCountries()
        }
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared
    ) {
        _path = path
        self.type = type
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager

        setup()
    }

    func connectionSelect(with country: Country) {
        switch type {
        case .entry:
            connectionManager.entryGateway = .country(code: country.code)
        case .exit:
            connectionManager.exitRouter = .country(code: country.code)
        }
        navigateHome()
    }

    func quickestConnectionSelect(with country: Country) {
        switch type {
        case .entry:
            connectionManager.entryGateway = .lowLatencyCountry(code: country.code)
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
            let newCountries: [Country]?
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

    func countriesMixnet() -> [Country] {
        switch type {
        case .entry:
            return !searchText.isEmpty ? countriesManager.entryCountries.filter {
                $0.name.lowercased().contains(searchText.lowercased()) ||
                $0.code.lowercased().contains(searchText.lowercased())
            } : countriesManager.entryCountries
        case .exit:
            return !searchText.isEmpty ? countriesManager.exitCountries.filter {
                $0.name.lowercased().contains(searchText.lowercased()) ||
                $0.code.lowercased().contains(searchText.lowercased())
            } : countriesManager.exitCountries
        }
    }

    func countriesWireGuard() -> [Country] {
        !searchText.isEmpty ? countriesManager.vpnCountries.filter {
            $0.name.lowercased().contains(searchText.lowercased()) ||
            $0.code.lowercased().contains(searchText.lowercased())
        } : countriesManager.vpnCountries
    }
}
