#if os(iOS)
import UIKit
#endif
import SwiftUI

@MainActor public final class ImpactGenerator: ObservableObject {
#if os(iOS)
    private let mediumFeedbackGenerator = UIImpactFeedbackGenerator(style: .medium)
    private let softFeedbackGenerator = UIImpactFeedbackGenerator(style: .soft)
    private let notificationGenerator = UINotificationFeedbackGenerator()
#endif
    public static let shared = ImpactGenerator()

    init() {
        setup()
    }

    public func softImpact() {
#if os(iOS)
        softFeedbackGenerator.impactOccurred()
#endif
    }

    public func impact() {
#if os(iOS)
        mediumFeedbackGenerator.impactOccurred()
#endif
    }

    public func success() {
#if os(iOS)
        notificationGenerator.notificationOccurred(.success)
#endif
    }

    public func error() {
#if os(iOS)
        notificationGenerator.notificationOccurred(.error)
#endif
    }

    public func warning() {
#if os(iOS)
        notificationGenerator.notificationOccurred(.warning)
#endif
    }
}

private extension ImpactGenerator {
    func setup() {
#if os(iOS)
        mediumFeedbackGenerator.prepare()
        softFeedbackGenerator.prepare()
        notificationGenerator.prepare()
#endif
    }
}
