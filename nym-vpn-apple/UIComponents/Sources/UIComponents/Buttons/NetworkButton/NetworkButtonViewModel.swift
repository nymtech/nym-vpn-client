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
    @Published var selectionImageColor: Color = NymColor.gray1
    @Published var selectionStrokeColor: Color = .clear

    var imageName: String {
        switch type {
        case .mixnet5hop:
            return "anonymous"
        case .wireguard:
            return "fast"
        }
    }

    var title: String {
        switch type {
        case .mixnet5hop:
            "5hopMixnetTitle".localizedString
        case .wireguard:
            "2hopMixnetTitle".localizedString
        }
    }

    var subtitle: String {
        switch type {
        case .mixnet5hop:
            "5hopMixnetSubtitle".localizedString
        case .wireguard:
            "2hopWireGuardSubtitle".localizedString
        }
    }

    func updateUI(isSelected: Bool) {
        self.selectionImageColor = isSelected ? NymColor.action : NymColor.gray1
        self.selectionStrokeColor = isSelected ? NymColor.action : .clear
    }
}
