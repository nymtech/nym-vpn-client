import SwiftUI
import CountriesManagerTypes
import Theme

public struct GatewayCellButtonViewModel {
    public enum GatewayyCellButtonType {
        case fastest(country: Country)
        case country(country: Country)
        case gateway(identifier: String)

        var country: Country? {
            switch self {
            case let .fastest(country), let .country(country):
                country
            case .gateway:
                nil
            }
        }
    }

    public let boltImageName = "bolt"
    public let selectedTitle = "selected".localizedString
    public let type: GatewayyCellButtonType
    public let isSelected: Bool

    public init(type: GatewayyCellButtonType, isSelected: Bool) {
        self.type = type
        self.isSelected = isSelected
    }

    public var title: String {
        switch type {
        case let .fastest(country):
            "fastest".localizedString + " (\(country.name))"
        case let .country(country):
            country.name
        case let .gateway(identifier):
            identifier
        }
    }
}
