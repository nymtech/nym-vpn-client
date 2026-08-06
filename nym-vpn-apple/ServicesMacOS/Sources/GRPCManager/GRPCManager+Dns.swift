import NymVPNLib

extension GRPCManager {
    public func getDefaultDns() async throws -> [IpAddr] {
        try await rpcClient?.getDefaultDns() ?? []
    }

    public func setEnableCustomDns(enable: Bool) async throws {
        try await rpcClient?.setEnableCustomDns(enable: enable)
    }

    public func setCustomDns(dnsServers: [IpAddr]) async throws {
        try await rpcClient?.setCustomDns(dnsServers: dnsServers)
    }
}
