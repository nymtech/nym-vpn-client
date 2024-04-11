import SwiftUI
import Theme

public struct NetworkButtonViewModel {
    public enum ButtonType {
        case mixnet5hop
        case mixnet2hop
        case wireguard

        var imageName: String {
            switch self {
            case .mixnet5hop:
                return "mixnetIcon"
            case .mixnet2hop:
                return "mixnetIcon"
            case .wireguard:
                return "wireguardIcon"
            }
        }

        var title: String {
            switch self {
            case .mixnet5hop:
                "5hopMixnetTitle".localizedString
            case .mixnet2hop:
                "2hopMixnetTitle".localizedString
            case .wireguard:
                "2hopWireGuardTitle".localizedString
            }
        }

        var subtitle: String {
            switch self {
            case .mixnet5hop:
                "5hopMixnetSubtitle".localizedString
            case .mixnet2hop:
                "2hopWireGuardSubtitle".localizedString
            case .wireguard:
                "2hopWireGuardSubtitle".localizedString
            }
        }
    }

    let type: ButtonType

    var isSmallScreen: Bool
    @Binding var selectedNetwork: ButtonType

    public init(type: ButtonType, selectedNetwork: Binding<ButtonType>, isSmallScreen: Bool = false) {
        self.type = type
        self._selectedNetwork = selectedNetwork
        self.isSmallScreen = isSmallScreen
    }

    private var isSelected: Bool {
        type == selectedNetwork
    }

    var selectionImageName: String {
        isSelected ? "networkSelectedCircle" : "networkCircle"
    }

    var selectionImageColor: Color {
        isSelected ? NymColor.primaryOrange : NymColor.networkButtonCircle
    }

    var selectionStrokeColor: Color {
        isSelected ? NymColor.primaryOrange : .clear
    }
}
