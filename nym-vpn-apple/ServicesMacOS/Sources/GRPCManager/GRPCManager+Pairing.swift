import NymVPNLib

extension GRPCManager {
    public func startPairing() async throws -> QrUrl {
        try await authorized { [weak self] in
            guard let rpcClient = await self?.rpcClient else {
                throw AccountDaemonRefreshError.clientUnavailable
            }
            return try await rpcClient.startPairing()
        }
    }

    public func verifiedPairingPin(sessionId: UInt32) async throws {
        try await authorized { [weak self] in
            guard let rpcClient = await self?.rpcClient else {
                throw AccountDaemonRefreshError.clientUnavailable
            }
            try await rpcClient.verifiedPairingPin(sessionId: sessionId)
        }
    }

    public func cancelPairingProcess() async throws {
        try await Task.detached { [weak self] in
            guard let rpcClient = await self?.rpcClient else {
                throw AccountDaemonRefreshError.clientUnavailable
            }
            try await rpcClient.cancelPairingProcess()
        }.value
    }
}
