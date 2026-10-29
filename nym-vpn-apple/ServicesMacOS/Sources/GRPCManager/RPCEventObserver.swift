import NymVPNLib
import Combine

final class RPCEventObserver: ObservableObject, EventObserver, @unchecked Sendable {
    let stream: AsyncStream<Event>
    let cont: AsyncStream<Event>.Continuation

    init() {
        let (stream, continuation) = AsyncStream<Event>.makeStream()

        self.stream = stream
        self.cont = continuation
    }

    func onEvent(event: Event) {
        cont.yield(event)
    }

    func onClose() {
        cont.finish()
    }
}
