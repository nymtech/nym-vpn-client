import Logging
import NetworkExtension
import MixnetLibrary
import NotificationMessages
import NymLogger
import Tunnels

actor TunnelActor {
    private let eventContinuation: AsyncStream<TunnelEvent>.Continuation
    private let defaultPathContinuation: AsyncStream<NWPath>.Continuation

    private let logger = Logger(label: "TunnelActor")

    weak var tunnelProvider: NEPacketTunnelProvider?

    var defaultPathObserver: (any OsDefaultPathObserver)?
    var defaultPathObservation: NSKeyValueObservation?

    /// Flag used to determine if `reasserting` property of tunnel provider can be used.
    /// Note that we shouldn't reassert unless we returned from `startTunnel()`
    var canReassert = false

    @Published private(set) var tunnelState: TunnelState?

    init() {
        let (eventStream, eventContinuation) = AsyncStream<TunnelEvent>.makeStream()
        self.eventContinuation = eventContinuation

        let (defaultPathStream, defaultPathContinuation) = AsyncStream<NWPath>.makeStream()
        self.defaultPathContinuation = defaultPathContinuation

        Task.detached { [weak self, eventStream] in
            for await case let .newState(tunnelState) in eventStream {
                await self?.setCurrentState(tunnelState)
            }
        }

        Task.detached { [weak self, defaultPathStream] in
            for await newPath in defaultPathStream {
                await self?.defaultPathObserver?.onDefaultPathChange(newPath: newPath.asOsDefaultPath())
            }
        }
    }

    deinit {
        eventContinuation.finish()
        defaultPathContinuation.finish()
    }

    nonisolated func onEvent(_ event: TunnelEvent) {
        eventContinuation.yield(event)
    }

    nonisolated func onDefaultPathChange(_ newPath: NWPath) {
        defaultPathContinuation.yield(newPath)
    }

    func setTunnelProvider(_ tunnelProvider: NEPacketTunnelProvider?) {
        self.tunnelProvider = tunnelProvider

        defaultPathObservation = tunnelProvider?.observe(\.defaultPath) { [weak self] tunnelProvider, _ in
            if let newPath = tunnelProvider.defaultPath {
                self?.onDefaultPathChange(newPath)
            }
        }
    }

    func setDefaultPathObserver(_ newObserver: (any OsDefaultPathObserver)?) {
        defaultPathObserver = newObserver
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

        case .error:
            if canReassert {
                // todo: remove once we properly handle error state
                // tunnelProvider?.cancelTunnelWithError(PacketTunnelProviderError.errorState)
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
            }
        }
    }
}
