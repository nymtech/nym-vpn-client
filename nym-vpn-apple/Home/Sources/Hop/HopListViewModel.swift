import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManager
import UIComponents

public class HopListViewModel: ObservableObject {
    let type: HopType
    let isSmallScreen: Bool

    var appSettings: AppSettings
    var connectionManager: ConnectionManager
    var countriesManager: CountriesManager
    @Binding var path: NavigationPath

    @Published var quickestCountry: Country?
    @Published var countries: [Country]?
    @Published var searchText: String = "" {
        didSet {
            updateQuickestCountry()
            updateCountries()
        }
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        isSmallScreen: Bool = false
    ) {
        _path = path
        self.type = type
        self.isSmallScreen = isSmallScreen
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
}

// MARK: - Navigation -
extension HopListViewModel {
    func navigateHome() {
        path = .init()
    }
}

// MARK: - Setup -
extension HopListViewModel {
    func setup() {
        updateQuickestCountry()
        updateCountries()
    }

    func updateQuickestCountry() {
        Task {
            guard type == .entry || (type == .exit && !appSettings.isEntryLocationSelectionOn),
                  let country = countriesManager.lowLatencyCountry
            else {
                quickestCountry = nil
                return
            }

            let newCountry: Country?
            if !searchText.isEmpty,
               !country.name.contains(searchText) {
                newCountry = nil
            } else {
                newCountry = country
            }

            Task { @MainActor in
                quickestCountry = newCountry
            }
        }
    }

    func updateCountries() {
        Task {
            let newCountries: [Country]?
            switch type {
            case .entry:
                newCountries = !searchText.isEmpty ? countriesManager.entryCountries?.filter {
                    $0.name.contains(
                        searchText
                    )
                } : countriesManager.entryCountries
            case .exit:
                newCountries = !searchText.isEmpty ? countriesManager.exitCountries?.filter {
                    $0.name.contains(
                        searchText
                    )
                } : countriesManager.exitCountries
            }
            Task { @MainActor in
                countries = newCountries
            }
        }
    }
}
