import Logging
import NetworkExtension
import ErrorReason
import MixnetLibrary
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
    }

    private func setCurrentState(_ state: TunnelState) async {
        switch state {
        case .connecting:
            if canReassert {
                tunnelProvider?.reasserting = true
            }
        case let .connected(connectionData):
            if canReassert {
                tunnelProvider?.reasserting = false
            }
            canReassert = true
            logger.info("🛤️ connected entry gw: \(connectionData.entryGateway), exit gw: \(connectionData.exitGateway)")
        case .error:
            if canReassert {
                // todo: remove once we properly handle error state
                tunnelProvider?.cancelTunnelWithError(PacketTunnelProviderError.errorState)
            }
        case .disconnecting(.error):
            await NotificationMessages.scheduleDisconnectNotification()
        default:
            break
        }

        tunnelState = state
    }

    /// Wait until the tunnel state shifted into either connected, disconnected or error state.
    func waitUntilStarted() async throws {
        var stateStream = $tunnelState.values.makeAsyncIterator()

        while case let .some(newState) = await stateStream.next() {
            switch newState {
            case .connected, .disconnected:
                return
            case let .error(errorStateReason):
                throw ErrorReason(with: errorStateReason).nsError
            case .disconnecting, .none, .connecting:
                break
            case let .some(.offline(reconnect: reconnect)):
                if reconnect {
                    break
                } else {
                    throw ErrorReason.offline
                }
            }
        }
    }
}
