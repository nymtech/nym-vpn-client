import Combine
import SwiftUI

public final class KeyboardManager: ObservableObject {
    private var cancellables = Set<AnyCancellable>()

    public static let shared = KeyboardManager()

    @Published public var change = KeyboardChange(height: 0, animation: .default)

    public init() {
        let willShow = NotificationCenter.default.publisher(for: UIResponder.keyboardWillShowNotification)
        let willChange = NotificationCenter.default.publisher(for: UIResponder.keyboardWillChangeFrameNotification)
        let willHide = NotificationCenter.default.publisher(for: UIResponder.keyboardWillHideNotification)

        willShow
            .merge(with: willChange)
            .merge(with: willHide)
            .map { notification -> KeyboardChange in
                let frame = (notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? CGRect) ?? .zero
                let isHiding = notification.name == UIResponder.keyboardWillHideNotification || frame.height == 0

                let duration = (notification.userInfo?[UIResponder.keyboardAnimationDurationUserInfoKey] as? Double)
                ?? (isHiding ? 0.16 : 0.25)

                return KeyboardChange(
                    height: isHiding ? 0 : frame.height,
                    animation: isHiding ? .easeOut(duration: duration) : .easeIn(duration: duration)
                )
            }
            .receive(on: RunLoop.main)
            .sink { [weak self] change in
                self?.change = change
            }
            .store(in: &cancellables)
    }

    public func hideKeyboard() {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}
