import SwiftUI
import ConnectionTypes
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayRandomCell: View {
    public enum Kind {
        case bestServer
        case random
    }

    private let hopType: HopType
    private let kind: Kind
    private let cornerRadius: CGFloat = 16

    @State private var isButtonHovered = false
    @Binding private var path: NavigationPath
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    private let algorithm: NymGatewaySelectionAlgorithm
    private let onTapOverride: (() -> Void)?

    private var isSelected: Bool {
        switch hopType {
        case .entry:
            if case .random = entryGateway { return true }
        case .exit:
            guard case .random = exitRouter else { return false }
            switch kind {
            case .bestServer:
                return algorithm == .auto
            case .random:
                return algorithm != .auto
            }
        }
        return false
    }

    public init(
        type: HopType,
        kind: Kind = .random,
        path: Binding<NavigationPath>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        algorithm: NymGatewaySelectionAlgorithm = .explicit,
        onTap: (() -> Void)? = nil
    ) {
        self.hopType = type
        self.kind = kind
        _path = path
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        self.algorithm = algorithm
        self.onTapOverride = onTap
    }

    private var iconName: String {
        switch kind {
        case .bestServer: "star.fill"
        case .random: "shuffle"
        }
    }

    private var iconTint: Color {
        switch kind {
        case .bestServer: .yellow
        case .random: Color.Nym.textPrimary
        }
    }

    private var labelKey: String {
        switch kind {
        case .bestServer: "gatewaysView.bestServer"
        case .random: "gatewaysView.random"
        }
    }

    public var body: some View {
        HStack(spacing: 0) {
            Image(systemName: iconName)
                .font(.system(size: 16, weight: .semibold))
                .foregroundStyle(iconTint)
                .frame(width: 24, height: 24)
                .padding(.leading, NymSpacing.large)
            Text(labelKey.localizedString)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.bodyLarge)
                .padding(.leading, NymSpacing.medium)
            Spacer()
        }
        .frame(height: 64)
        .frame(maxWidth: .infinity)
        .contentShape(Rectangle())
        .background(isButtonHovered ? Color.Nym.surfaceBg.opacity(0.3) : Color.clear)
        .background {
            RoundedRectangle(cornerRadius: cornerRadius)
                .fill(Color.Nym.surfaceElev)
        }
        .overlay {
            RoundedRectangle(cornerRadius: cornerRadius)
                .inset(by: 0.5)
                .stroke(isSelected ? Color.Nym.brandPrimary : .clear, lineWidth: 1)
                .allowsHitTesting(false)
        }
        .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
        .padding(.horizontal, NymSpacing.large)
        .padding(.bottom, NymSpacing.small)
        .animation(.default, value: isSelected)
        .accessibilityElement(children: .combine)
        .accessibilityLabel(labelKey.localizedString)
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

private extension GatewayRandomCell {
    func tapAction() {
        ImpactGenerator.shared.softImpact()
        if let onTapOverride {
            onTapOverride()
        } else {
            switch hopType {
            case .entry:
                entryGateway = .random
            case .exit:
                exitRouter = .random
            }
        }
        path = .init()
    }
}
