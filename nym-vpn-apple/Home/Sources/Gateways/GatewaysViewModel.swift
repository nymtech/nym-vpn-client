import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManagerTypes
import GatewayManager
import UIComponents

@MainActor public class GatewaysViewModel: ObservableObject {
    let gatewayManager: GatewayManager
    let type: HopType
    let minimumSearchSymbols = 2

    @ObservedObject var connectionManager: ConnectionManager
    @Binding var path: NavigationPath
    @Published var isGeolocationModalDisplayed = false
    @Published var gateways = [GatewayNode]()
    @Published var countries = [NymCountry]()
    @Published var foundCountries = [NymCountry]()
    @Published var foundUSRegions = [String]()
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

extension GatewaysViewModel {
    func gatewaysInCountry(with countryCode: String) -> [GatewayNode] {
        gateways.filter {
            $0.location?.twoLetterIsoCountryCode.caseInsensitiveCompare(countryCode) == .orderedSame
        }
    }
}

// MARK: - Setup -
private extension GatewaysViewModel {
    func setup() {
        updateGateways()
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
        Task { [weak self] in
            guard let self else { return }
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
            let result = Array(Set(gateways.map { $0.location?.twoLetterIsoCountryCode }))
                .compactMap { self.gatewayManager.localizedCountry(with: $0) }
                .sorted {
                    $0.name.compare(
                        $1.name,
                        options: [.caseInsensitive, .diacriticInsensitive, .widthInsensitive],
                        range: nil,
                        locale: Locale.current
                    ) == .orderedAscending
                }
            await MainActor.run {
                self.countries = result
            }
        }
    }

    func searchCountriesGateways() {
        Task { [weak self] in
            guard let self, searchText.count >= minimumSearchSymbols
            else {
                await MainActor.run {
                    self?.foundCountries = [NymCountry]()
                    self?.foundGateways = [GatewayNode]()
                }
                return
            }
            let newCountries = countries.filter {
                $0.name.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased())
                || $0.code.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased())
            }

            var seen = Set<String>()
            let newRegions = gateways
                .filter { $0.location?.twoLetterIsoCountryCode.caseInsensitiveCompare("US") == .orderedSame }
                .compactMap { $0.location?.region.trimmingCharacters(in: .whitespacesAndNewlines) }
                .filter {
                    !$0.isEmpty
                    && $0.range(of: self.searchText, options: [.caseInsensitive, .diacriticInsensitive]) != nil
                }
                .filter { seen.insert($0).inserted }

            let newGateways = gateways.filter {
                $0.name?.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased()) ?? false
                || $0.id.lowercased().localizedCaseInsensitiveContains(self.searchText.lowercased())
            }
            await MainActor.run {
                self.foundCountries = newCountries
                self.foundUSRegions = newRegions
                self.foundGateways = newGateways
            }
        }
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
