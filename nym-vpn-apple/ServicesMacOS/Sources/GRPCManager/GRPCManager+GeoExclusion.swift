import NymVPNLib

extension GRPCManager {
    public func setGeoExclusionEnabled(_ enabled: Bool) async throws {
        try await rpcClient?.setGeoExclusionEnabled(enabled: enabled)
    }

    public func setGeoExclusionListenPort(_ port: UInt16) async throws {
        try await rpcClient?.setGeoExclusionListenPort(listenPort: port)
    }

    public func setGeoExclusionExcludedCountries(_ countries: [String]) async throws {
        try await rpcClient?.setGeoExclusionExcludedCountries(excludedCountries: countries)
    }
}
