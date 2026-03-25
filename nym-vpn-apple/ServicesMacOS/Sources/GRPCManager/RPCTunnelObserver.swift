import NymVPNRpc
import Combine
import Logging

final class RPCTunnelObserver: ObservableObject, TunnelEventObserver, @unchecked Sendable {
    public let stream: AsyncStream<TunnelEvent>
    let cont: AsyncStream<TunnelEvent>.Continuation

    init() {
        let (stream, continuation) = AsyncStream<TunnelEvent>.makeStream()

        self.stream = stream
        self.cont = continuation
    }

    lazy var logger = Logger(label: "RPCTunnelObserver")

    func onTunnelEvent(event: TunnelEvent) {
        cont.yield(event)
    }

    func onClose() {
        cont.finish()
    }
}
