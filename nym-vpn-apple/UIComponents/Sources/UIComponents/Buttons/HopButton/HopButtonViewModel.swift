import SwiftUI
import Combine
import AppSettings
import ConfigurationManager
import ConnectionManager
import CountriesManager

public class HopButtonViewModel: ObservableObject {
    private let appSettings: AppSettings
    let connectionManager: ConnectionManager
    let arrowImageName = "arrowRight"
    let hopType: HopType

    var name: String {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.name
        case .exit:
            connectionManager.exitRouter.name
        }
    }

    var isQuickest: Bool {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.isQuickest
        case .exit:
            false
        }
    }

    var countryCode: String? {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.countryCode
        case .exit:
            connectionManager.exitRouter.countryCode
        }
    }

    var isGateway: Bool {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.isGateway
        case .exit:
            connectionManager.exitRouter.isGateway
        }
    }

    public init(
        hopType: HopType,
        appSettings: AppSettings = .shared,
        connectionManager: ConnectionManager = .shared
    ) {
        self.hopType = hopType
        self.appSettings = appSettings
        self.connectionManager = connectionManager
    }
}
