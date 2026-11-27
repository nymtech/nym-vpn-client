extension GRPCManager {
    public func deviceIdentifier() async throws -> String? {
        try await client.getDeviceIdentity()
    }

    public func accountIdentifier() async throws -> String? {
        try await client.getAccountIdentity()
    }
}
