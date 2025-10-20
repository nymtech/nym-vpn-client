import NetworkExtension

@MainActor extension NETunnelProviderManager {
    public func saveToPreferencesAndLoadTunnels() async throws {
        try await saveToPreferences()
        try await loadFromPreferences()
    }

    func savePrefsAndReloadOnMainActor() async throws {
        try await saveToPreferencesAndLoadTunnels()
    }
}
