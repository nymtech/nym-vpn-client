import Logging
import NetworkExtension
import ErrorReason
import NymVPNLib
import NotificationMessages
import NymLogger
import Tunnels

actor TunnelActor {
    private let eventContinuation: AsyncStream<TunnelEvent>.Continuation

    private let logger = Logger(label: "TunnelActor")

    weak var tunnelProvider: NEPacketTunnelProvider?

    /// Flag used to determine if `reasserting` property of tunnel provider can be used.
    /// Note that we shouldn't reassert unless we returned from `startTunnel()`
    var canReassert = false

    @Published private(set) var tunnelState: TunnelState?
    var lastError: ErrorReason?

    /// Suspends `startTunnel` while the gateway-independence pre-flight waits
    /// for the user to accept relaxed criteria (resumed by the
    /// `setGatewayIndependence(false)` app message).
    private var relaxConsentContinuation: CheckedContinuation<Void, Error>?

    init() {
        let (eventStream, eventContinuation) = AsyncStream<TunnelEvent>.makeStream()
        self.eventContinuation = eventContinuation

        Task.detached { [weak self, eventStream] in
            for await case let .newState(tunnelState) in eventStream {
                await self?.setCurrentState(tunnelState)
            }
        }
    }

    deinit {
        eventContinuation.finish()
    }

    nonisolated func onEvent(_ event: TunnelEvent) {
        eventContinuation.yield(event)
    }

    func setTunnelProvider(_ tunnelProvider: NEPacketTunnelProvider?) {
        self.tunnelProvider = tunnelProvider

        if let provider = tunnelProvider as? PacketTunnelProvider,
           let failure = provider.logInitFailure {
            lastError = .createLogFailed(failure)
        }
    }

    private func setCurrentState(_ state: TunnelState) async {
        switch state {
        case .connecting:
            if canReassert {
                tunnelProvider?.reasserting = true
            }
        case .connected:
            if canReassert {
                tunnelProvider?.reasserting = false
            }
            canReassert = true
        case let .error(errorStateReason):
            if canReassert, errorStateReason != .needsRelaxedIndependenceCriteria {
                tunnelProvider?.cancelTunnelWithError(PacketTunnelProviderError.errorState)
            }
            lastError = ErrorReason(with: errorStateReason)
            tunnelState = .error(errorStateReason)
            return
        case .disconnecting(.error):
            await NotificationMessages.scheduleDisconnectNotification()
        default:
            break
        }

        tunnelState = state
    }

    // MARK: - Gateway-independence pre-flight

    func reportNeedsRelaxedIndependence() {
        lastError = .needsRelaxedIndependenceCriteria
        tunnelState = .error(.needsRelaxedIndependenceCriteria)
    }

    func awaitRelaxConsent() async throws {
        try await withCheckedThrowingContinuation { continuation in
            relaxConsentContinuation = continuation
        }
    }

    /// Resumes a pending `awaitRelaxConsent()`. Returns `true` if a pre-flight
    /// wait was released (suspend path), `false` when none was pending (error
    /// path: library already errored, so the caller must reconnect explicitly).
    @discardableResult
    func resumeRelaxConsent() -> Bool {
        guard let continuation = relaxConsentContinuation else { return false }
        continuation.resume()
        relaxConsentContinuation = nil
        return true
    }

    /// Aborts a pending `awaitRelaxConsent()` on tunnel teardown by throwing,
    /// so the suspended `startTunnel` unwinds instead of falling through to
    /// `connectTunnel`. No-op if nothing is waiting.
    func cancelRelaxConsent() {
        relaxConsentContinuation?.resume(throwing: CancellationError())
        relaxConsentContinuation = nil
    }

    func clearError() {
        lastError = nil
    }

    /// Wait until the tunnel state shifted into either connected, disconnected or error state.
    func waitUntilStarted() async throws {
        var stateStream = $tunnelState.values.makeAsyncIterator()

        while case let .some(newState) = await stateStream.next() {
            switch newState {
            case .connected, .disconnected:
                return
            case let .error(errorStateReason):
                lastError = ErrorReason(with: errorStateReason)
            case .disconnecting, .none, .connecting:
                break
            case let .some(.offline(reconnect: reconnect)):
                if reconnect {
                    break
                } else {
                    throw ErrorReason.offline.nsError
                }
            }
        }
    }
}
