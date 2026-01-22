import SwiftUI

public struct KeyboardHostView<Content: View>: View {
    private let content: Content
    private let bottomSafeAreaInset: CGFloat

    @EnvironmentObject private var keyboardManager: KeyboardManager
    @State private var keyboardHeight: CGFloat = 0

    public init(
        bottomSafeAreaInset: CGFloat,
        @ViewBuilder _ content: @escaping () -> Content
    ) {
        self.bottomSafeAreaInset = bottomSafeAreaInset
        self.content = content()
    }

    public var body: some View {
        content
            .onReceive(keyboardManager.$change) { change in
                DispatchQueue.main.async {
                    withAnimation(change.animation) {
                        keyboardHeight = change.height
                    }
                }
            }
            .padding(.bottom, max(0, keyboardHeight - bottomSafeAreaInset))
    }
}
