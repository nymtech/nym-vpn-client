#if os(iOS)
import NymVPNLib

/// Forwards the account controller events, such as QR pairing results, into an async stream.
final class AccountControllerEventListener: StatusListener {
    let stream: AsyncStream<Event>
    private let cont: AsyncStream<Event>.Continuation

    init() {
        // Bounded, so the events don't pile up while nothing consumes the stream
        let (stream, continuation) = AsyncStream<Event>.makeStream(bufferingPolicy: .bufferingNewest(16))

        self.stream = stream
        self.cont = continuation
    }

    deinit {
        cont.finish()
    }

    func onEvent(event: Event) {
        cont.yield(event)
    }
}
#endif
