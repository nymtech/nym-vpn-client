import ConnectionTypes
import NymVPNLib

extension GRPCManager {
    public func getDefaultDns() async throws -> [CustomDnsConfig.Server] {
        try await rpcClient?.getDefaultDns().map { dns in CustomDnsConfig.Server(from: dns) } ?? []
    }

    public func setEnableCustomDns(enable: Bool) async throws {
        try await rpcClient?.setEnableCustomDns(enable: enable)
    }

    public func setCustomDns(dnsServers: [IpAddr]) async throws {
        try await rpcClient?.setCustomDns(dnsServers: [DnsServer](plainDnsAddresses: dnsServers))
    }
}
