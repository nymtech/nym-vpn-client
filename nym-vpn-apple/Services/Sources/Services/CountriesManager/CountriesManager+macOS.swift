#if os(macOS)
import Foundation
import Combine
import Constants

// MARK: - Fetching -
extension CountriesManager {
    func fetchEntryExitCountries() {
        fetchCountries(uri: Constants.entryCountries.rawValue)
            .sink(receiveCompletion: { [weak self] completion in
                self?.isLoading = false

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
                self?.entryLastHopStore.exitCountries = countries
                self?.entryLastHopStore.lastFetchDate = Date()
                self?.exitCountries = countries
            })
            .store(in: &cancellables)
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
#endif
