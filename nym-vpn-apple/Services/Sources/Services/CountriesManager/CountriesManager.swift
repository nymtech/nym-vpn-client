import Combine
import SwiftUI
import AppSettings
import Constants
import Logging

public final class CountriesManager: ObservableObject {
    private var appSettings: AppSettings

    let logger = Logger(label: "CountriesManager")

    var isLoading = false
    var entryLastHopStore = EntryLastHopStore()
    var cancellables = Set<AnyCancellable>()

    public static let shared = CountriesManager(appSettings: AppSettings.shared)

    @Published public var entryCountries: [Country]
    @Published public var exitCountries: [Country]
    @Published public var lastError: Error?

    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
        self.entryCountries = []
        self.exitCountries = []

        setup()
    }

    public func fetchCountries() {
        guard !isLoading, needsReload(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
        else {
            loadTemporaryCountries(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
            return
        }
        isLoading = true

        Task {
            fetchEntryExitCountries()
        }
    }

    public func country(with code: String, isEntryHop: Bool) -> Country? {
        if isEntryHop {
            return entryCountries.first(where: { $0.code == code })
        } else {
            return exitCountries.first(where: { $0.code == code })
        }
    }
}

// MARK: - Setup -
private extension CountriesManager {
    func setup() {
        loadPrebundledCountries()
        setupAppSettingsObservers()
        fetchCountries()
    }

    func setupAppSettingsObservers() {
        appSettings.$isEntryLocationSelectionOnPublisher.sink { [weak self] _ in
            self?.fetchCountries()
        }
        .store(in: &cancellables)
    }
}

// MARK: - Pre bundled countries -
private extension CountriesManager {
    func loadPrebundledCountries() {
        guard let entryCountriesURL = Bundle.main.url(forResource: "gatewaysEntryCountries", withExtension: "json"),
              let exitCountriesURL = Bundle.main.url(forResource: "gatewaysExitCountries", withExtension: "json")
        else {
            updateError(with: GeneralNymError.noPrebundledCountries)
            return
        }

        do {
            let prebundledEntryCountries = try loadPrebundledCountries(from: entryCountriesURL)
            let prebundledExitCountries = try loadPrebundledCountries(from: exitCountriesURL)

            entryLastHopStore.entryCountries = prebundledEntryCountries
            entryLastHopStore.exitCountries = prebundledExitCountries

            entryCountries = prebundledEntryCountries
            exitCountries = prebundledExitCountries
        } catch let error {
            updateError(with: error)
            return
        }
    }

    func loadPrebundledCountries(from fileURL: URL) throws -> [Country] {
        do {
            let data = try Data(contentsOf: fileURL)
            let countryCodes = try JSONDecoder().decode([String].self, from: data)
            let countries = countryCodes.compactMap { [weak self] countryCode in
                self?.country(with: countryCode)
            }
            .sorted(by: { $0.name < $1.name })

            return countries
        } catch {
            throw GeneralNymError.cannotParseCountries
        }
    }
}

private extension CountriesManager {
    // TODO: extract API layer to service
    func fetchEntryExitCountries() {
        let dispatchGroup = DispatchGroup()

        dispatchGroup.enter()
        fetchCountries(uri: Constants.entryCountries.rawValue)
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { [weak self] completion in
                dispatchGroup.leave()

                switch completion {
                case .finished:
                    break
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
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { [weak self] completion in
                dispatchGroup.leave()

                switch completion {
                case .finished:
                    break
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
}

private extension CountriesManager {
    func fetchCountries(uri: String) -> AnyPublisher<[Country], Error> {
        guard let url = URL(string: uri)
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
        guard let lastFetchDate = entryLastHopStore.lastFetchDate else { return true }
        return isLongerThan10Minutes(date: lastFetchDate)
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
            exitCountries = entryLastHopStore.exitCountries
            entryCountries = entryLastHopStore.entryCountries
        }
    }
}

// MARK: - Helper -
extension CountriesManager {
    func updateError(with error: Error) {
        lastError = error
    }
}
