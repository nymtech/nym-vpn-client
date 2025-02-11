import GRPC

extension GRPCManager {
    func setupHealthObserver() {
        let call = healthClient.watch(Grpc_Health_V1_HealthCheckRequest()) { [weak self] response in
            let result = response.status == .serving

            guard self?.isServing != result else { return }
            self?.isServing = result
            self?.requiredCallsAfterSeviceIsUp()
        }

        call.status.whenComplete { [weak self] result in
            switch result {
            case .success(let status):
                if status.code == .unavailable {
                    self?.tunnelStatus = .disconnected
                    self?.isServing = false
                    self?.daemonVersion = "unknown"
                    self?.setup()
                }
            case let .failure(error):
                self?.logger.error("\(error)")
            }
        }
    }
}

private extension GRPCManager {
    func requiredCallsAfterSeviceIsUp() {
        guard isServing else { return }

        Task(priority: .background) { [weak self] in
            _ = try? await self?.version()
        }
    }
}
