import SwiftUI

extension View {
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
