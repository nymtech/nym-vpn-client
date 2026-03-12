import Foundation
import ConnectionTypes
import UIComponents

public enum HomeLink: Hashable, Identifiable, Codable {
    case gatewayDetails(gateway: GatewayNode, hopType: HopType)
    case entryGateways
    case exitGateways
    case settings
    case launchView
    case onboarding
    case technicalOptIns

    public var id: String {
        String(describing: self)
    }
}
