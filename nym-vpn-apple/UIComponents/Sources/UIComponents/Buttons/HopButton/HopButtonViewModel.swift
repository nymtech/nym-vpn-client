import SwiftUI
import Combine
import AppSettings
import ConfigurationManager
import ConnectionManager
import CountriesManager

public class HopButtonViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
    private let connectionManager: ConnectionManager
    private let countriesManager: CountriesManager

    private var cancellables = Set<AnyCancellable>()

    let arrowImageName = "arrowRight"
    let hopType: HopType

    @Published var gateway: String?
    @Published var countryCode: String?
    @Published var countryName: String?
    @Published var isQuickest = false

    public init(
        hopType: HopType,
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        connectionManager: ConnectionManager = .shared,
        countriesManager: CountriesManager = .shared
    ) {
        self.hopType = hopType
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager

        setupConnectionManagerObserver()
    }
}

private extension HopButtonViewModel {
    func setupConnectionManagerObserver() {
        connectionManager.$entryGateway.sink { [weak self] _ in
            guard self?.hopType == .entry else { return }
            self?.updateData()
        }
        .store(in: &cancellables)

        connectionManager.$exitRouter.sink { [weak self] _ in
            guard self?.hopType == .exit else { return }
            self?.updateData()
        }
        .store(in: &cancellables)
    }
}

private extension HopButtonViewModel {
    func updateData() {
        if configurationManager.isSantaClaus {
            var newGateway: String?
            switch hopType {
            case .entry:
                if let gateway = appSettings.entryGateway {
                    newGateway = gateway
                }
            case .exit:
                if let gateway = appSettings.exitGateway {
                    newGateway = gateway
                }
            }
            guard let newGateway
            else {
                gateway = nil
                updateCountryData()
                return
            }
            gateway = newGateway
        } else {
            updateCountryData()
        }
    }
    func updateCountryData() {
        Task { @MainActor in
            let text: String?
            switch hopType {
            case .entry:
                isQuickest = connectionManager.entryGateway.isQuickest
                countryCode = connectionManager.entryGateway.countryCode
                switch connectionManager.connectionType {
                case .mixnet5hop:
                    text = countryName(for: countryCode ?? "", countryType: .entry)
                case .wireguard:
                    text = countryName(for: countryCode ?? "", countryType: .vpn)
                }
            case .exit:
                isQuickest = connectionManager.exitRouter.isQuickest
                countryCode = connectionManager.exitRouter.countryCode
                switch connectionManager.connectionType {
                case .mixnet5hop:
                    text = countryName(for: countryCode ?? "", countryType: .exit)
                case .wireguard:
                    text = countryName(for: countryCode ?? "", countryType: .vpn)
                }
            }

            guard let text
            else {
                // Get country even if it is not in the list...
                if let fallbackCountry = countriesManager.country(with: countryCode ?? "") {
                    let name = fallbackCountry.name
                    return countryName = isQuickest ? "fastest".localizedString + " (\(name))" : name
                }
                return countryName = isQuickest ? "fastest".localizedString : nil
            }
            countryName = isQuickest ? "fastest".localizedString + " (\(text))" : text
        }
    }

    func updateSantaData() {
        Task { @MainActor in
            switch hopType {
            case .entry:
                gateway = appSettings.entryGateway
            case .exit:
                gateway = appSettings.exitGateway
            }
        }
    }

    func countryName(for countryCode: String, countryType: CountryType) -> String? {
        switch countryType {
        case .entry:
            countriesManager.country(with: countryCode, countryType: .entry)?.name
        case .exit:
            countriesManager.country(with: countryCode, countryType: .exit)?.name
        case .vpn:
            countriesManager.country(with: countryCode, countryType: .vpn)?.name
        }
    }
}
