import SwiftUI
import Theme

public struct CopyButton: View {
    private let onCopy: () -> Void

    @State private var didCopy = false

    public var body: some View {
        GenericImage(imageName: didCopy ? "checkmarkSeeThrough" : "copy")
            .accessibilityElement(children: .combine)
            .accessibilityLabel("accessibility.doubleTap.copy".localizedString)
            .accessibilityAddTraits([.isButton])
            .animation(.easeInOut, value: didCopy)
            .foregroundStyle(NymColor.gray1)
            .frame(width: 20, height: 20)
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
