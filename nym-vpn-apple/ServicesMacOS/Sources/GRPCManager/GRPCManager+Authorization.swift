import NymVPNLib

let kAuthorizationPrompt = "NymVPN needs to access your account."

extension GRPCManager {
    /// Runs an account request guarded by the daemon. When the daemon requires user presence,
    /// asks the user to authenticate and retries the request once.
    ///
    /// Skips the prompt if another request authorized the daemon while this one was in flight.
    @discardableResult
    func authorized<T: Sendable>(_ request: @escaping @Sendable () async throws -> T) async throws -> T {
        let generation = authorizationGeneration
        do {
            return try await request()
        } catch let error as RpcError where error.isUserPresenceRequired() {
            if generation == authorizationGeneration {
                try await authorize()
            }
            return try await request()
        }
    }
}

private extension GRPCManager {
    /// Concurrent requests share a single authentication prompt.
    func authorize() async throws {
        if let authorizationTask {
            return try await authorizationTask.value
        }

        let manager = try authorizationManager ?? AuthorizationManager()
        authorizationManager = manager

        let task = Task.detached { [weak self] in
            let token = try await manager.authorize(prompt: kAuthorizationPrompt)
            guard let rpcClient = await self?.rpcClient else {
                throw AccountDaemonRefreshError.clientUnavailable
            }
            try await rpcClient.authorize(token: token)
        }
        authorizationTask = task
        defer { authorizationTask = nil }

        try await task.value
        authorizationGeneration += 1
    }
}
