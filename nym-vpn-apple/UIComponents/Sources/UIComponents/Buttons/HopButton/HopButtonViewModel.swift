import SwiftUI
import Combine
import AppSettings
import ConfigurationManager
import ConnectionManager
import ConnectionTypes
import CountriesManager

public class HopButtonViewModel: ObservableObject {
    private let appSettings: AppSettings

    let arrowImageName = "arrowRight"
    let hopType: HopType

    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter

    var name: String {
        switch hopType {
        case .entry:
            entryGateway.name
        case .exit:
            exitRouter.name
        }
    }

    var isQuickest: Bool {
        switch hopType {
        case .entry:
            entryGateway.isQuickest
        case .exit:
            false
        }
    }

    var countryCode: String? {
        switch hopType {
        case .entry:
            entryGateway.countryCode
        case .exit:
            exitRouter.countryCode
        }
    }

    var isGateway: Bool {
        switch hopType {
        case .entry:
            entryGateway.isGateway
        case .exit:
            exitRouter.isGateway
        }
    }

    public init(
        hopType: HopType,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        appSettings: AppSettings = .shared
    ) {
        self.hopType = hopType
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        self.appSettings = appSettings
    }
}
