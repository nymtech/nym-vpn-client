import SwiftUI
import Theme

public struct CopyButton: View {
    private let onCopy: () -> Void

    @State private var didCopy = false

    public var body: some View {
        GenericImage(imageName: didCopy ? "checkmarkSeeThrough" : "copy")
            .animation(.easeInOut, value: didCopy)
            .foregroundStyle(Color.Nym.textSecondary)
            .frame(width: 20, height: 20)
            .padding(12)
            .contentShape(Rectangle())
            .accessibilityElement(children: .combine)
            .accessibilityLabel("accessibility.doubleTap.copy".localizedString)
            .accessibilityAddTraits([.isButton])
            .onTapGesture {
                copyAction()
            }
            .accessibilityAction {
                copyAction()
            }
    }

    public init(onCopy: @escaping () -> Void) {
        self.onCopy = onCopy
    }
}

private extension CopyButton {
    func copyAction() {
        onCopy()
        withAnimation(.easeInOut) {
            didCopy = true
        }

        Task {
            try? await Task.sleep(for: .seconds(3))
            withAnimation(.easeInOut) {
                didCopy = false
            }
        }
    }
}
