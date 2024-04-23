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
    }
}

private extension HopButtonViewModel {
    func setupConnectionManagerObserver() {
        connectionManager.$entryGateway.sink { [weak self] _ in
            self?.updateData()
        }
        .store(in: &cancellables)

        connectionManager.$exitRouter.sink { [weak self] _ in
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
                isQuickest = connectionManager.entryGateway?.isQuickest ?? false
                countryCode = connectionManager.entryGateway?.countryCode
                text = countriesManager.country(with: countryCode ?? "", isEntryHop: hopType == .entry)?.name
            case .exit:
                isQuickest = connectionManager.exitRouter?.isQuickest ?? false
                countryCode = connectionManager.exitRouter?.countryCode
                text = countriesManager.country(with: countryCode ?? "", isEntryHop: false)?.name
            }

            guard let text else { return countryName = isQuickest ? "fastest".localizedString : nil }
            countryName = isQuickest ? "fastest".localizedString + " (\(text))" : text
        }
    }
}
