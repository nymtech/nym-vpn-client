extension GRPCManager {
    public func deleteLog() async throws {
        try await rpcClient?.deleteLogFile()
    }
}
