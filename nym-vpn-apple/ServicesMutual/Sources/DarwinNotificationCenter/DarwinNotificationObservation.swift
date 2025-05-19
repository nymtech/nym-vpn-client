import Foundation
import Combine

/// Object that retains an observation of Darwin notifications.
///
/// Retain this object to keep the observer active.
///
/// Save this object in a variable, or store it in a bag.
///
/// ```
/// observation.store(in: &disposeBag)
/// ```
///
/// To stop observing the notifiation, deallocate the this object, or call the `cancel()` method.
public final class DarwinNotificationObservation: Cancellable {
    // Wrapper class around the callback closure.
    // This object can stay alive in the cancel block, after this Observation has been deallocated.
    class Closure {
        let invoke: () -> Void
        init(callback: @escaping () -> Void) {
            self.invoke = callback
        }
    }
    let closure: Closure

    init(callback: @escaping () -> Void) {
        self.closure = Closure(callback: callback)
    }

    deinit {
        cancel()
    }

    /// Cancels the Darwin notification observation.
    public func cancel() {
        // Notifications are always delivered on the main thread.
        // So we also remove the observer on the main thread,
        // to make sure the closure object isn't deallocated during the execution of a notification.
        DispatchQueue.main.async { [closure] in
            let pointer = UnsafeRawPointer(Unmanaged.passUnretained(closure).toOpaque())
            CFNotificationCenterRemoveObserver(CFNotificationCenterGetDarwinNotifyCenter(), pointer, nil, nil)
        }
    }
}
