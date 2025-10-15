extension GRPCManager {
    public func deleteLog() async throws {
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.deleteLogFile()
        }.value
    }
}
