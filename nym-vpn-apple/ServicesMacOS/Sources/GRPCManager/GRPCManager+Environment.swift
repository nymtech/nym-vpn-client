extension GRPCManager {
    public func switchEnvironment(to environment: String) async throws {
        logger.info("Changing env to \(environment)")
        try await Task.detached { [weak self] in
            try await self?.rpcClient?.setNetwork(network: environment)
        }.value
    }
}
