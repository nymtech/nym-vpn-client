#if os(iOS)
import NymVPNLib

/// Forwards the account controller events, such as QR pairing results, into an async stream.
final class AccountControllerEventListener: TunnelStatusListener {
    let stream: AsyncStream<TunnelEvent>
    private let cont: AsyncStream<TunnelEvent>.Continuation

    init() {
        // Bounded, so the events don't pile up while nothing consumes the stream
        let (stream, continuation) = AsyncStream<TunnelEvent>.makeStream(bufferingPolicy: .bufferingNewest(16))

        self.stream = stream
        self.cont = continuation
    }

    deinit {
        cont.finish()
    }

    func onEvent(event: TunnelEvent) {
        cont.yield(event)
    }
}
#endif
