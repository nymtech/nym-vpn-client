import SwiftUI

public struct KeyboardHostView<Content: View>: View {
    private let content: Content
    private let bottomSafeAreaInset: CGFloat
    private let isEnabled: Bool

    @EnvironmentObject private var keyboardManager: KeyboardManager
    @State private var keyboardHeight: CGFloat = 0

    public init(
        bottomSafeAreaInset: CGFloat,
        isEnabled: Bool = true,
        @ViewBuilder _ content: @escaping () -> Content
    ) {
        self.bottomSafeAreaInset = bottomSafeAreaInset
        self.isEnabled = isEnabled
        self.content = content()
    }

    public var body: some View {
        content
            .onReceive(keyboardManager.$change) { change in
                guard isEnabled else { return }
                DispatchQueue.main.async {
                    withAnimation(change.animation) {
                        keyboardHeight = change.height
                    }
                }
            }
            .onChange(of: isEnabled) { _, newValue in
                guard !newValue, keyboardHeight != 0 else { return }
                withAnimation(.easeOut(duration: 0.16)) {
                    keyboardHeight = 0
                }
            }
            .padding(.bottom, max(0, keyboardHeight - bottomSafeAreaInset))
    }
}
