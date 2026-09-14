import SwiftUI
import Theme

/// One row for a built-in connection profile — shared by the Connect screen's quick picker and the Settings list.
public struct ProfileRow: View {
    private let imageName: String
    private let title: String
    private let description: String
    private let isSelected: Bool
    private let onTap: () -> Void

    @State private var isHovered = false

    public init(
        imageName: String,
        title: String,
        description: String,
        isSelected: Bool,
        onTap: @escaping () -> Void
    ) {
        self.imageName = imageName
        self.title = title
        self.description = description
        self.isSelected = isSelected
        self.onTap = onTap
    }

    public var body: some View {
        HStack(alignment: .top, spacing: NymSpacing.medium) {
            GenericImage(imageName: imageName)
                .foregroundStyle(isSelected ? Color.Nym.primary : Color.Nym.textSecondary)
                .frame(width: Constants.iconSize, height: Constants.iconSize)
                .padding(.top, 2)
            VStack(alignment: .leading, spacing: NymSpacing.extraExtraSmall) {
                Text(title)
                    .nymTextStyle(.bodyDefaultBold)
                    .foregroundStyle(isSelected ? Color.Nym.primary : Color.Nym.textPrimary)
                Text(description)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textSecondary)
            }
            Spacer()
        }
        .padding(.horizontal, NymSpacing.medium)
        .padding(.vertical, NymSpacing.small)
        .contentShape(Rectangle())
        .background(rowBackground)
        .clipShape(RoundedRectangle(cornerRadius: Constants.cornerRadius))
        .onHover { isHovered = $0 }
        .onTapGesture(perform: onTap)
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits(.isButton)
        .accessibilityValue(isSelected ? "selected".localizedString : "")
    }

    private var rowBackground: Color {
        if isSelected {
            Color.Nym.primary.opacity(0.1)
        } else if isHovered {
            Color.Nym.surfacePressed
        } else {
            Color.clear
        }
    }

    private enum Constants {
        static let iconSize: CGFloat = 20
        static let cornerRadius: CGFloat = 12
    }
}
