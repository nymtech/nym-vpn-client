import SwiftUI
import AppSettings
import UIComponents
import Tunnels

public class HomeViewModel: HomeFlowState {
    private let appSettings: AppSettings
    private let tunnelsManager: TunnelsManager
    private let screenSize: CGSize

    @Published var selectedNetwork: NetworkButtonViewModel.ButtonType

    public init(
        screenSize: CGSize,
        selectedNetwork: NetworkButtonViewModel.ButtonType,
        appSettings: AppSettings = AppSettings.shared,
        tunnelsManager: TunnelsManager = TunnelsManager.shared
    ) {
        self.selectedNetwork = selectedNetwork
        self.appSettings = appSettings
        self.tunnelsManager = tunnelsManager
        self.screenSize = screenSize

        tunnelsManager.loadConfigurations()
    }
}

// MARK: - Navigation -

public extension HomeViewModel {
    func navigateToSettings() {
        path.append(HomeLink.settings)
    }

    func navigateToFirstHopSelection() {
        path.append(HomeLink.firstHop(text: ""))
    }

    func navigateToLastHopSelection() {
        path.append(HomeLink.lastHop)
    }
}

// MARK: - Helpers -

public extension HomeViewModel {
    func isSmallScreen() -> Bool {
        screenSize.width <= 375 && screenSize.height <= 647
    }

    func shouldShowEntryHop() -> Bool {
        appSettings.entryLocationSelectionIsOn
    }
}

// MARK: - Tunnel testing -

public extension HomeViewModel {
    func connect() {
        if let tunnel = tunnelsManager.currentTunnel, tunnel.tunnel.connection.status == .connected {
            tunnelsManager.disconnect()
        } else {
            tunnelsManager.test()
        }
    }
}
