import UIKit

public final class ImpactGenerator {
    private let feedbackGenerator = UIImpactFeedbackGenerator(style: .medium)
    private let notificationGenerator = UINotificationFeedbackGenerator()

    public static let shared = ImpactGenerator()

    init() {
        setup()
    }

    public func impact() {
        feedbackGenerator.impactOccurred()
    }

    public func success() {
        notificationGenerator.notificationOccurred(.success)
    }

    public func error() {
        notificationGenerator.notificationOccurred(.error)
    }

    public func warning() {
        notificationGenerator.notificationOccurred(.warning)
    }
}

private extension ImpactGenerator {
    func setup() {
        feedbackGenerator.prepare()
        notificationGenerator.prepare()
    }
}
