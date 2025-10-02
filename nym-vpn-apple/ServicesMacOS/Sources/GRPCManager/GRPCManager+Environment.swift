extension GRPCManager {
    public func switchEnvironment(to environment: String) async throws {
        logger.info("Changing env to \(environment)")
        try await rpcClient?.setNetwork(network: environment)
    }
}
