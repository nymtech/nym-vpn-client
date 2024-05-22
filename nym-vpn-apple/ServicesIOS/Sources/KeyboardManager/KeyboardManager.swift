import Combine
import SwiftUI

public final class KeyboardManager: ObservableObject {
    private let keyboardWillShowNotification = UIResponder.keyboardWillShowNotification
    private let keyboardWillChangeFrameNotification = UIResponder.keyboardWillChangeFrameNotification
    private let keyboardWillHideNotification = UIResponder.keyboardWillHideNotification
    private var cancellables = Set<AnyCancellable>()

    public static let shared = KeyboardManager()

    @Published public var change = KeyboardChange(height: 0, animation: .default)

    public init() {
        NotificationCenter.Publisher(center: .default, name: keyboardWillShowNotification)
            .merge(with: NotificationCenter.Publisher(center: .default, name: keyboardWillChangeFrameNotification))
            .merge(
                with:
                    NotificationCenter.Publisher(center: .default, name: keyboardWillHideNotification)
                    .map {
                        Notification(name: $0.name, object: $0.object, userInfo: nil)
                    }
            )
            .map { notification -> KeyboardChange in
                let frame = notification.userInfo?[UIWindow.keyboardFrameEndUserInfoKey] as? CGRect ?? .zero
                let isHiding = frame.size.height == .zero
                let defaultDuration = isHiding ? 0.16 : 0.25
                let durationValue = notification.userInfo?[UIWindow.keyboardAnimationDurationUserInfoKey] as? Double
                let duration = durationValue ?? defaultDuration
                return KeyboardChange(
                    height: frame.size.height,
                    animation: isHiding ? .easeOut(duration: duration) : .easeIn(duration: duration)
                )
            }
            .assign(to: \.change, on: self)
            .store(in: &cancellables)
    }
}
