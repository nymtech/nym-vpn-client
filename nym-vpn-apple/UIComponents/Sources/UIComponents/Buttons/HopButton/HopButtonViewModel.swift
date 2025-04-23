import SwiftUI
import Combine
import AppSettings
import ConfigurationManager
import ConnectionManager
import ConnectionTypes
import CountriesManager
import Localizations

public class HopButtonViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let countriesManager: CountriesManager

    private var cancellables = Set<AnyCancellable>()
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter

    let arrowImageName = "arrowRight"
    let hopType: HopType

    var name: String {
        let countryCode: String
        switch hopType {
        case .entry:
            countryCode = entryGateway.name
        case .exit:
            countryCode = exitRouter.name
        }
        return countriesManager.localizedCountry(with: countryCode)?.name ?? countryCode
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
        appSettings: AppSettings = .shared,
        countriesManager: CountriesManager = .shared
    ) {
        self.hopType = hopType
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        self.appSettings = appSettings
        self.countriesManager = countriesManager
    }
}
