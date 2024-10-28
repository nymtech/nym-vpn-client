extension GRPCManager {
    public func switchEnvironment(to environment: String) throws {
        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            throw GRPCError.daemonNotRunning
        }
        logger.info("Changing env to \(environment)")

        var request = Nym_Vpn_SetNetworkRequest()
        request.network = environment

        let call = client.setNetwork(request)

        call.response.whenComplete { [weak self] result in
            switch result {
            case .success(let response):
                self?.logger.error("\(response.error.message)")
            case .failure(let error):
                self?.logger.error("\(error)")
            }
        }

        do {
            _ = try call.status.wait()
        }
    }
}
