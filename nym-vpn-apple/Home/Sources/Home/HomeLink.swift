import Foundation
import CountriesManagerTypes
import UIComponents

enum HomeLink: Hashable, Identifiable {
    case gatewayDetails(gateway: GatewayNode, hopType: HopType)
    case entryGateways
    case exitGateways
    case settings


    var id: String {
        String(describing: self)
    }
}
