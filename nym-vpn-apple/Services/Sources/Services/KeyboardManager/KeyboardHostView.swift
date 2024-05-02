import SwiftUI

public struct KeyboardHostView<Content>: View  where Content: View {
    private let content: Content

    @EnvironmentObject private var keyboardManager: KeyboardManager
    @State private var keyboardHeight = 0.0

    public init(@ViewBuilder _ content: @escaping () -> Content) {
        self.content = content()
    }

    public var body: some View {
        content
            .onReceive(keyboardManager.$change) { change in
                withAnimation(change.animation) {
                    keyboardHeight = change.height
                }
            }
            .padding(.bottom, keyboardHeight)
    }
}
