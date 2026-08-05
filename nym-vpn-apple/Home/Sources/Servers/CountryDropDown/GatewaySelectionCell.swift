import SwiftUI
import ImpactGenerator
import Theme
import UIComponents

/// Shared layout for the "Random"/"Safest" rows
struct GatewaySelectionCell: View {
    let imageName: String
    let titleKey: String
    let isSelected: Bool
    let onTap: () -> Void

    private let cornerRadius: CGFloat = 16
    @State private var isButtonHovered = false

    var body: some View {
        HStack(spacing: 0) {
            GenericImage(imageName: imageName)
                .foregroundStyle(Color.Nym.textPrimary)
                .frame(width: 24, height: 24)
                .padding(.leading, NymSpacing.large)
            Text(titleKey.localizedString)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyLarge)
                .padding(.leading, NymSpacing.medium)
            Spacer()
        }
        .frame(height: 64)
        .frame(maxWidth: .infinity)
        .contentShape(Rectangle())
        .background(isButtonHovered ? Color.Nym.background.opacity(0.3) : Color.clear)
        .background {
            RoundedRectangle(cornerRadius: cornerRadius)
                .fill(Color.Nym.surface)
        }
        .overlay {
            RoundedRectangle(cornerRadius: cornerRadius)
                .inset(by: 0.5)
                .stroke(isSelected ? Color.Nym.primary : .clear, lineWidth: 1)
                .allowsHitTesting(false)
        }
        .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
        .padding(.horizontal, NymSpacing.large)
        .padding(.bottom, NymSpacing.small)
        .animation(.default, value: isSelected)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(titleKey.localizedString)
        .accessibilityValue(isSelected ? "selected".localizedString : "")
        .accessibilityAddTraits([.isButton])
        .onHover { newValue in
            isButtonHovered = newValue
        }
        .onTapGesture {
            ImpactGenerator.shared.softImpact()
            onTap()
        }
        .accessibilityAction {
            ImpactGenerator.shared.softImpact()
            onTap()
        }
    }
}
