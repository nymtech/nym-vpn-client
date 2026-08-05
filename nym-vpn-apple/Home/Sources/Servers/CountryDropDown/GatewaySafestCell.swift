import SwiftUI
import ConnectionTypes
import ImpactGenerator
import Theme
import UIComponents

public struct GatewaySafestCell: View {
    private let hopType: HopType
    private let cornerRadius: CGFloat = 16

    @State private var isButtonHovered = false
    @Binding private var path: NavigationPath
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    private let onTapOverride: (() -> Void)?

    private var isSelected: Bool {
        switch hopType {
        case .entry:
            if case .auto = entryGateway { return true }
        case .exit:
            if case .auto = exitRouter { return true }
        }
        return false
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        onTap: (() -> Void)? = nil
    ) {
        self.hopType = type
        _path = path
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        self.onTapOverride = onTap
    }

    public var body: some View {
        HStack(spacing: 0) {
            GenericImage(imageName: "safest")
                .foregroundStyle(Color.Nym.textPrimary)
                .frame(width: 24, height: 24)
                .padding(.leading, NymSpacing.large)
            Text("gatewaysView.safest".localizedString)
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
        .accessibilityLabel("gatewaysView.safest".localizedString)
        .accessibilityValue(isSelected ? "selected".localizedString : "")
        .accessibilityAddTraits([.isButton])
        .onHover { newValue in
            isButtonHovered = newValue
        }
        .onTapGesture {
            tapAction()
        }
        .accessibilityAction {
            tapAction()
        }
    }
}

private extension GatewaySafestCell {
    func tapAction() {
        ImpactGenerator.shared.softImpact()
        if let onTapOverride {
            onTapOverride()
        } else {
            switch hopType {
            case .entry:
                entryGateway = .auto
            case .exit:
                exitRouter = .auto
            }
        }
        path = .init()
    }
}
