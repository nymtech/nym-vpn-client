extension GRPCManager {
    public func deviceIdentifier() async throws -> String? {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.getDeviceIdentity()
        }.value
    }

    public func accountIdentifier() async throws -> String? {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.getAccountIdentity()
        }.value
    }
}
