import SwiftUI
import ConnectionManager
import CountriesManager
import UIComponents

public class HopListViewModel: ObservableObject {
    let type: HopType
    let isSmallScreen: Bool

    var connectionManager: ConnectionManager
    var countriesManager: CountriesManager
    @Binding var path: NavigationPath

    @Published var quickestCountry: Country?
    @Published var countries: [Country]?
    @Published var searchText: String = ""

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared,
        isSmallScreen: Bool = false
    ) {
        _path = path
        self.type = type
        self.isSmallScreen = isSmallScreen
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
        guard type == .entry,
              let country = countriesManager.lowLatencyCountry
        else {
            quickestCountry = nil
            return
        }

        if !searchText.isEmpty,
           !country.name.contains(searchText) {
            print("contains")
            quickestCountry = nil
        } else {
            quickestCountry = country
        }
    }

    func updateCountries() {
        switch type {
        case .entry:
            countries = !searchText.isEmpty ? countriesManager.entryCountries?.filter {
                $0.name.contains(
                    searchText
                )
            } : countriesManager.entryCountries
        case .exit:
            countries = !searchText.isEmpty ? countriesManager.exitCountries?.filter {
                $0.name.contains(
                    searchText
                )
            } : countriesManager.exitCountries
        }
    }
}
