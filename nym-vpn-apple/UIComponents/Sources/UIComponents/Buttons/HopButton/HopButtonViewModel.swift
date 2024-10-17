import SwiftUI
import Combine
import ConnectionManager
import CountriesManager

public class HopButtonViewModel: ObservableObject {
    private var connectionManager: ConnectionManager
    private var countriesManager: CountriesManager
    private var cancellables = Set<AnyCancellable>()

    let arrowImageName = "arrowRight"
    let hopType: HopType

    @Published var countryCode: String?
    @Published var countryName: String?
    @Published var isQuickest = false

    public init(
        hopType: HopType,
        connectionManager: ConnectionManager = ConnectionManager.shared,
        countriesManager: CountriesManager = CountriesManager.shared
    ) {
        self.hopType = hopType
        self.connectionManager = connectionManager
        self.countriesManager = countriesManager

        setupConnectionManagerObserver()
        updateData()
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
                return countryName = isQuickest ? "fastest".localizedString : nil
            }
            countryName = isQuickest ? "fastest".localizedString + " (\(text))" : text
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
