import Foundation
import Combine

/// Wrapper around the application’s Darwin notification center from CFNotificationCenter.h
///
/// - Note: On macOS, consider using DistributedNotificationCenter instead
public final class DarwinNotificationCenter {
    private init() {}

    /// The application’s Darwin notification center.
    public static var shared = DarwinNotificationCenter()

    /// Posts a Darwin notification with the specified name.
    public func post(name: String) {
        CFNotificationCenterPostNotification(
            CFNotificationCenterGetDarwinNotifyCenter(),
            CFNotificationName(rawValue: name as CFString),
            nil,
            nil,
            true
        )
    }

    /// Registers an observer closure for Darwin notifications of the specified name.
    ///
    /// Retain the returned `DarwinNotificationObservation` to keep the observer active.
    ///
    /// Save the returned value in a variable, or store it in a bag.
    ///
    /// ```
    /// observation.store(in: &disposeBag)
    /// ```
    ///
    /// To stop observing the notifiation, deallocate the `DarwinNotificationObservation`, or call its `cancel()` method.
    public func addObserver(name: String, callback: @escaping () -> Void) -> DarwinNotificationObservation {
        let observation = DarwinNotificationObservation(callback: callback)

        let pointer = UnsafeRawPointer(Unmanaged.passUnretained(observation.closure).toOpaque())

        CFNotificationCenterAddObserver(
            CFNotificationCenterGetDarwinNotifyCenter(),
            pointer,
            notificationCallback,
            name as CFString,
            nil,
            .deliverImmediately
        )

        return observation
    }
}

private func notificationCallback(
    center: CFNotificationCenter?,
    observation: UnsafeMutableRawPointer?,
    name: CFNotificationName?,
    object _: UnsafeRawPointer?,
    userInfo _: CFDictionary?
) {
    guard let pointer = observation else { return }
    let closure = Unmanaged<DarwinNotificationObservation.Closure>.fromOpaque(pointer).takeUnretainedValue()
    closure.invoke()
}

// MARK: - AsyncSequence -

extension DarwinNotificationCenter {

    /// Returns an asynchronous sequence of notifications for a given notification name.
    func notifications(named name: String) -> AsyncStream<Void> {
        AsyncStream { continuation in
            let observation = addObserver(name: name) {
                continuation.yield()
            }
            continuation.onTermination = { _ in
                observation.cancel()
            }
        }
    }
}

// MARK: - Combine -

#if canImport(Combine)
extension DarwinNotificationCenter {
    /// Returns a publisher that emits events when broadcasting notifications.
    ///
    /// - Parameters:
    ///   - name: The name of the notification to publish.
    /// - Returns: A publisher that emits events when broadcasting notifications.
    public func publisher(for name: String) -> DarwinNotificationCenter.Publisher {
        Publisher(center: self, name: name)
    }
}

extension DarwinNotificationCenter {
    /// A publisher that emits when broadcasting notifications.
    public struct Publisher: Combine.Publisher {
        public typealias Output = Void
        public typealias Failure = Never
        public let center: DarwinNotificationCenter
        public let name: String
        public init(center: DarwinNotificationCenter, name: String) {
            self.center = center
            self.name = name
        }

        public func receive<S>(subscriber: S) where S: Subscriber, S.Failure == Never, S.Input == Output {
            let observation = center.addObserver(name: name) {
                _ = subscriber.receive()
            }

            subscriber.receive(subscription: observation)
        }
    }
}

extension DarwinNotificationObservation: Subscription {
    public func request(_ demand: Subscribers.Demand) {}
}
#endif
