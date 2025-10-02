extension GRPCManager {
    public func deviceIdentifier() async throws -> String? {
        try await rpcClient?.getDeviceIdentity()
    }

    public func accountIdentifier() async throws -> String? {
        try await rpcClient?.getAccountIdentity()
    }
}
