import SwiftUI

/// Blocked: the lit ring blinks slowly between 0.15 and 0.9 opacity over
/// 2.4 s while the kill switch holds traffic. Still at 0.9 under Reduce Motion,
/// and while the scene is not active or the marker is off screen (no timeline runs then).
struct MarkerBlink: ViewModifier {
    static let period: TimeInterval = 2.4
    static let low = 0.15
    static let high = 0.9

    let isActive: Bool

    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var isOnScreen = true

    func body(content: Content) -> some View {
        Group {
            if !isActive {
                content
            } else if reduceMotion || scenePhase != .active || !isOnScreen {
                content.opacity(Self.high)
            } else {
                TimelineView(.animation(minimumInterval: 1.0 / 30.0)) { context in
                    content.opacity(Self.opacity(at: context.date.timeIntervalSinceReferenceDate))
                }
            }
        }
        .onAppear { isOnScreen = true }
        .onDisappear { isOnScreen = false }
    }

    /// Eases from `low` up to `high` and back once per period.
    static func opacity(at time: TimeInterval) -> Double {
        let phase = time.truncatingRemainder(dividingBy: period) / period
        return low + (high - low) * (1 - cos(2 * .pi * phase)) / 2
    }
}
