import SwiftUI
import AppSettings

public final class CountriesManager: ObservableObject {
    private var appSettings: AppSettings

    var isLoading = false
    var lastHopStore = LastHopStore(lastFetchDate: Date())
    var entryLastHopStore = EntryLastHopStore(lastFetchDate: Date())

    public static let shared = CountriesManager(appSettings: AppSettings.shared)

    @Published public var entryCountries: [Country]?
    @Published public var exitCountries: [Country]?
    @Published public var lowLatencyCountry: Country?
    @Published public var hasCountries = false

    public init(appSettings: AppSettings) {
        self.appSettings = appSettings
    }

    public func fetchCountries() throws {
        guard !isLoading, needReload(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
        else {
            loadTemporaryCountries(shouldFetchEntryCountries: appSettings.isEntryLocationSelectionOn)
            return
        }
        isLoading = true

        Task {
            if appSettings.isEntryLocationSelectionOn {
                try fetchEntryExitCountries()
                fetchLowLatencyEntryCountry()
            } else {
                try fetchExitCountries()
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

// MARK: - Temp storage -
private extension CountriesManager {
    func needReload(shouldFetchEntryCountries: Bool) -> Bool {
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
                lowLatencyCountry = entryLastHopStore.lowLatencyCountry
            } else {
                exitCountries = lastHopStore.countries
                entryCountries = nil
                lowLatencyCountry = nil
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
}
