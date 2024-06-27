import SwiftUI
import Combine
import AppSettings
import ConnectionManager
import Theme

public final class NetworkButtonViewModel: ObservableObject {
    let type: ConnectionType

    private let appSettings: AppSettings
    private let connectionManager: ConnectionManager
    private var cancellables = Set<AnyCancellable>()

    public init(
        type: ConnectionType,
        appSettings: AppSettings = AppSettings.shared,
        connectionManager: ConnectionManager = ConnectionManager.shared
    ) {
        self.type = type
        self.connectionManager = connectionManager
        self.appSettings = appSettings

        self.isSmallScreen = appSettings.isSmallScreen

        connectionManager.$connectionType.sink { [weak self] newType in
            let isSelected = newType == self?.type
            self?.updateUI(isSelected: isSelected)
        }
        .store(in: &cancellables)
    }

    @Published var isSmallScreen: Bool
    @Published var selectionImageName: String = "networkCircle"
    @Published var selectionImageColor: Color = NymColor.networkButtonCircle
    @Published var selectionStrokeColor: Color = .clear

    var imageName: String {
        switch type {
        case .mixnet5hop:
            return "mixnetIcon"
        case .mixnet2hop:
            return "mixnetIcon"
        case .wireguard:
            return "wireguardIcon"
        }
    }

    var title: String {
        switch type {
        case .mixnet5hop:
            "5hopMixnetTitle".localizedString
        case .mixnet2hop:
            "2hopMixnetTitle".localizedString
        case .wireguard:
            "2hopWireGuardTitle".localizedString
        }
    }

    var subtitle: String {
        switch type {
        case .mixnet5hop:
            "5hopMixnetSubtitle".localizedString
        case .mixnet2hop:
            "2hopWireGuardSubtitle".localizedString
        case .wireguard:
            "2hopWireGuardSubtitle".localizedString
        }
    }

    func updateUI(isSelected: Bool) {
        self.selectionImageName = isSelected ? "networkSelectedCircle" : "networkCircle"
        self.selectionImageColor = isSelected ? NymColor.primaryOrange : NymColor.networkButtonCircle
        self.selectionStrokeColor = isSelected ? NymColor.primaryOrange : .clear
    }
}
