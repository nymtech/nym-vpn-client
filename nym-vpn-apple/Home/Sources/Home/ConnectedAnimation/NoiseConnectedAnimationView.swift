import SwiftUI
import AppSettings
import ConnectionManager
import UIComponents

struct NoiseConnectedAnimationView: View {
    @Environment(\.colorScheme)
    private var colorScheme

    @EnvironmentObject private var connectionManager: ConnectionManager

    var body: some View {
        if connectionManager.currentTunnelStatus == .connected {
            LoopAnimationView(
                animationName: NoiseConnectedAnimation(
                    isDarkMode: isDarkMode(),
                    isMixnet: connectionManager.connectionType == .mixnet5hop
                ).rawValue
            )
        }
    }
}

private extension NoiseConnectedAnimationView {
    func isDarkMode() -> Bool {
        colorScheme == .dark
    }
}

enum NoiseConnectedAnimation: String {
    case darkWireguard = "connected-2hop-dark"
    case lightWireguard = "connected-2hop-light"
    case darkMixnet = "connected-5hop-dark"
    case lightMixnet = "connected-5hop-light"

    init (isDarkMode: Bool, isMixnet: Bool) {
        self = isDarkMode ? isMixnet ? .darkMixnet : .darkWireguard : isMixnet ? .lightMixnet : .lightWireguard
    }
}
