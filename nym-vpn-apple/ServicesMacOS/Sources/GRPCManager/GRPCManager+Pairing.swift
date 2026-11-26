import Foundation
import NymVPNLib

extension GRPCManager {
    /// A stream per subscriber, since `AsyncStream` supports one consumer. The daemon doesn't replay events.
    public func subscribeForPairingEvents() -> AsyncStream<PairingEvent> {
        let (stream, continuation) = AsyncStream.makeStream(of: PairingEvent.self, bufferingPolicy: .bufferingNewest(16))
        let id = UUID()
        pairingSubscribers[id] = continuation
        continuation.onTermination = { [weak self] _ in
            Task { @MainActor in self?.pairingSubscribers[id] = nil }
        }
        return stream
    }

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
