import Combine
import SwiftUI
import AppSettings
import Constants
import Logging

public final class CountriesManager: ObservableObject {
    private var appSettings: AppSettings

    let logger = Logger(label: "CountriesManager")

    var isLoading = false
    var lastHopStore = LastHopStore(lastFetchDate: Date())
    var entryLastHopStore = EntryLastHopStore(lastFetchDate: Date())
    var cancellables = Set<AnyCancellable>()

    public static let shared = CountriesManager(appSettings: AppSettings.shared)

    @Published public var entryCountries: [Country]?
    @Published public var exitCountries: [Country]?
    @Published public var hasCountries = false
    @Published public var lastError: Error?

    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }

    public func fetchCountries() throws {
        guard !isLoading, needsReload(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
        else {
            loadTemporaryCountries(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
            return
        }
        isLoading = true

        Task {
            if appSettings.isEntryLocationSelectionOn {
                fetchEntryExitCountries()
            } else {
                fetchExitCountries()
            }
        }
    }

    public func country(with code: String, isEntryHop: Bool) -> Country? {
        if isEntryHop {
            return entryCountries?.first(where: { $0.code == code })
        } else {
            return exitCountries?.first(where: { $0.code == code })
        }
    }
}

private extension CountriesManager {
    // TODO: extrac API layer to service
    func fetchEntryExitCountries() {
        let dispatchGroup = DispatchGroup()

        dispatchGroup.enter()
        fetchCountries(uri: Constants.entryCountries.rawValue)
            .sink(receiveCompletion: { [weak self] completion in
                dispatchGroup.leave()

                switch completion {
                case .finished:
                    self?.updateHasCountries()
                case .failure:
                    self?.updateError(with: GeneralNymError.cannotFetchCountries)
                }
            }, receiveValue: { [weak self] countries in
                self?.entryLastHopStore.entryCountries = countries
                self?.entryLastHopStore.lastFetchDate = Date()
                self?.entryCountries = countries
            })
            .store(in: &cancellables)

        dispatchGroup.enter()
        fetchCountries(uri: Constants.exitCountries.rawValue)
            .sink(receiveCompletion: { [weak self] completion in
                dispatchGroup.leave()

                switch completion {
                case .finished:
                    self?.updateHasCountries()
                case .failure:
                    self?.updateError(with: GeneralNymError.cannotFetchCountries)
                }
            }, receiveValue: { [weak self] countries in
                self?.entryLastHopStore.exitCountries = countries
                self?.entryLastHopStore.lastFetchDate = Date()
                self?.exitCountries = countries
            })
            .store(in: &cancellables)

        dispatchGroup.notify(queue: .main) { [weak self] in
            self?.isLoading = false
        }
    }

    func fetchExitCountries() {
        fetchCountries(uri: Constants.exitCountries.rawValue)
            .sink(receiveCompletion: { [weak self] completion in
                self?.isLoading = false

                switch completion {
                case .finished:
                    self?.updateHasCountries()
                case .failure:
                    self?.updateError(with: GeneralNymError.cannotFetchCountries)
                }
            }, receiveValue: { [weak self] countries in
                self?.lastHopStore.countries = countries
                self?.lastHopStore.lastFetchDate = Date()
                self?.entryCountries = nil
                self?.exitCountries = countries
            })
            .store(in: &cancellables)
    }
}

private extension CountriesManager {
    func fetchCountries(uri: String) -> AnyPublisher<[Country], Error> {
        guard let url = URL(string: Constants.exitCountries.rawValue)
        else {
            return Fail(error: GeneralNymError.invalidUrl).eraseToAnyPublisher()
        }

        return URLSession.shared.dataTaskPublisher(for: URLRequest(url: url))
            .map { $0.data }
            .decode(type: [String].self, decoder: JSONDecoder())
            .map { [weak self] values in
                values.compactMap { self?.country(with: $0) }
            }
            .map { countries in
                countries.sorted(by: { $0.name < $1.name })
            }
            .eraseToAnyPublisher()
    }

    func country(with countryCode: String) -> Country? {
        guard let countryName = Locale.current.localizedString(forRegionCode: countryCode)
        else {
            logger.log(level: .error, "Failed resolving country code for: \(countryCode)")
            return nil
        }
        return Country(name: countryName, code: countryCode)
    }
}

// MARK: - Temp storage -
private extension CountriesManager {
    func needsReload(shouldFetchEntryCountries: Bool) -> Bool {
        if shouldFetchEntryCountries {
            guard let countries = entryLastHopStore.entryCountries, !countries.isEmpty else { return true }
        } else {
            guard let countries = lastHopStore.countries, !countries.isEmpty else { return true }
        }

        if shouldFetchEntryCountries {
            let lastFetchDate = entryLastHopStore.lastFetchDate
            return isLongerThan10Minutes(date: lastFetchDate)
        } else {
            let lastFetchDate = lastHopStore.lastFetchDate
            return isLongerThan10Minutes(date: lastFetchDate)
        }
    }

    func isLongerThan10Minutes(date: Date) -> Bool {
        let difference = Date().timeIntervalSince(date)
        if difference > 600 {
            return true
        } else {
            return false
        }
    }

    func loadTemporaryCountries(shouldFetchEntryCountries: Bool) {
        Task { @MainActor in
            if shouldFetchEntryCountries {
                exitCountries = entryLastHopStore.exitCountries
                entryCountries = entryLastHopStore.entryCountries
            } else {
                exitCountries = lastHopStore.countries ?? entryLastHopStore.exitCountries
                entryCountries = nil
            }
            updateHasCountries()
        }
    }
}

// MARK: - Helper -
extension CountriesManager {
    func updateHasCountries() {
        if appSettings.isEntryLocationSelectionOn {
            hasCountries = ((entryCountries?.isEmpty) != nil)
        } else {
            hasCountries = ((exitCountries?.isEmpty) != nil)
        }
    }

    func updateError(with error: Error) {
        lastError = error
    }
}
