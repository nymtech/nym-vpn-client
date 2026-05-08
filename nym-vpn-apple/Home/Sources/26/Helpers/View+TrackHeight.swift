import SwiftUI

extension View {
    /// Reports the view's rendered height via `action` whenever it changes,
    /// using a background GeometryReader. Height changes propagate without
    /// their own animation transaction so the caller's enclosing
    /// `.animation(value:)` modifier drives the transition cleanly.
    func trackHeight(_ action: @escaping (CGFloat) -> Void) -> some View {
        background(
            GeometryReader { geo in
                Color.clear
                    .onAppear { action(geo.size.height) }
                    .onChange(of: geo.size.height) { _, newHeight in action(newHeight) }
            }
        )
    }
}
