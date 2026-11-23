import SwiftUI

/// Blocked: the outer ring's gap turns clockwise once every 4 s while the kill
/// switch holds traffic. Still (gap ending at 12 o'clock) under Reduce Motion,
/// and while the scene is not active or the marker is off screen (no timeline runs then).
struct MarkerOrbit: ViewModifier {
    static let period: TimeInterval = 4

    let isActive: Bool

    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @Environment(\.scenePhase) private var scenePhase
    @State private var isOnScreen = true

    func body(content: Content) -> some View {
        // the hooks sit on a stable ZStack, not on the branches, so a branch swap cannot flip isOnScreen
        ZStack {
            if !isActive || reduceMotion || scenePhase != .active || !isOnScreen {
                content
            } else {
                TimelineView(.animation(minimumInterval: 1.0 / 30.0)) { context in
                    content.rotationEffect(.degrees(Self.angle(at: context.date.timeIntervalSinceReferenceDate)))
                }
            }
        }
        .onAppear { isOnScreen = true }
        .onDisappear { isOnScreen = false }
    }

    /// Degrees turned, 0 up to 360, once per period.
    static func angle(at time: TimeInterval) -> Double {
        time.truncatingRemainder(dividingBy: period) / period * 360
    }
}
