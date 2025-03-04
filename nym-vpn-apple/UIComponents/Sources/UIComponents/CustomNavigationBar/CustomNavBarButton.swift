import SwiftUI
import Theme

public struct CustomNavBarButton: View {
    public enum ButtonType: String {
        case back
        case settings
        case info
        case empty

        var imageName: String? {
            switch self {
            case .back:
                "arrowBack"
            case .settings:
                "settingsGear"
            case .info:
                "info"
            case .empty:
                nil
            }
        }
    }
    @State private var isHovered = false

    public let type: ButtonType
    public let action: (() -> Void)?

    public init(type: ButtonType, action: (() -> Void)?) {
        self.type = type
        self.action = action
    }

    public var body: some View {
        if #available(iOS 17.0, macOS 14.0, *) {
            genericButton()
                .focusEffectDisabled()
        } else {
            genericButton()
#if os(macOS)
                .focusable(false)
#endif
        }
    }
}

private extension CustomNavBarButton {
    @ViewBuilder
    func genericButton() -> some View {
        Button {
            action?()
        } label: {
            if let imageName = type.imageName {
                Image(imageName, bundle: .module)
                    .foregroundStyle(NymColor.navigationBarSettingsGear)
            }
        }
        .buttonStyle(PlainButtonStyle())
        .frame(width: 48, height: 48)
        .onHover { newValue in
            isHovered = newValue
        }
        .opacity(isHovered ? 0.7 : 1)
    }
}
