import NetworkExtension

extension NETunnelProviderManager {
    public func saveToPreferencesAndLoadTunnels() async throws {
        try await saveToPreferences()
        try await loadFromPreferences()
    }
}
