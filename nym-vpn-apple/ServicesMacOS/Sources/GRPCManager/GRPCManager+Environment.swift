extension GRPCManager {
    public func switchEnvironment(to environment: String) async throws {
        logger.info("Changing env to \(environment)")
        try await client.setNetwork(network: environment)
    }
}
