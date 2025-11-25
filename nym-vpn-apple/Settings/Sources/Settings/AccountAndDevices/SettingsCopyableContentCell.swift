import SwiftUI
import Theme
import UIComponents

public struct SettingsCopyableContentCell: View {
    private let title: String
    private let subtitle: String
    private let systemImageName: String
    private let imageSize: CGFloat
    private let onCopy: () -> Void

    @State private var didCopy = false

    public var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 16) {
                GenericImage(systemImageName: systemImageName)
                    .frame(width: imageSize, height: imageSize)
                    .foregroundStyle(NymColor.gray1)

                Text(title)
                    .textStyle(.Body.Large.regular)
                    .foregroundStyle(NymColor.primary)
            }
            .accessibilityElement(children: .combine)
            .accessibilityAddTraits([.isStaticText])
            .accessibilityLabel(title)

            HStack(alignment: .top, spacing: 16) {
                Text(subtitle)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)

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
        }
        .frame(maxWidth: .infinity)
        .padding(16)
        .background(NymColor.elevation)
        .cornerRadius(8)
    }

    public init(
        title: String,
        subtitle: String,
        systemImageName: String,
        imageSize: CGFloat,
        onCopy: @escaping () -> Void
    ) {
        self.title = title
        self.subtitle = subtitle
        self.systemImageName = systemImageName
        self.imageSize = imageSize
        self.onCopy = onCopy
    }
}

extension SettingsCopyableContentCell {
    func copyAction() {
        onCopy()
        withAnimation(.easeInOut) {
            didCopy = true
        }

        didCopy = true
        Task {
            try? await Task.sleep(for: .seconds(3))
            withAnimation(.easeInOut) {
                didCopy = false
            }
        }
    }
}
