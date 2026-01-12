import Foundation
import CountriesManagerTypes
import UIComponents

public enum HomeLink: Hashable, Identifiable, Codable {
    case gatewayDetails(gateway: GatewayNode, hopType: HopType)
    case entryGateways
    case exitGateways
    case settings
    case launchView
    case onboarding

    public var id: String {
        String(describing: self)
    }
}
