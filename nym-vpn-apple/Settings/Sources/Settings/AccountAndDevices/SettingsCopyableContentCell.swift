import SwiftUI
import Theme
import UIComponents

public struct SettingsCopyableContentCell: View {
    private let title: String
    private let subtitle: String
    private let systemImageName: String
    private let imageSize: CGFloat
    private let onCopy: () -> Void

    public var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 16) {
                GenericImage(systemImageName: systemImageName)
                    .frame(width: imageSize, height: imageSize)
                    .foregroundStyle(Color.Nym.textSecondary)

                Text(title)
                    .nymTextStyle(.bodyLarge)
                    .foregroundStyle(Color.Nym.textPrimary)
            }
            .accessibilityElement(children: .combine)
            .accessibilityAddTraits([.isStaticText])
            .accessibilityLabel(title)

            HStack(alignment: .top, spacing: 16) {
                Text(subtitle)
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .fixedSize(horizontal: false, vertical: true)
                    .layoutPriority(1)

                CopyButton(onCopy: onCopy)
            }
        }
        .frame(maxWidth: .infinity)
        .padding(16)
        .background(Color.Nym.surfaceElev)
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
