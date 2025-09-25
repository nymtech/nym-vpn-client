import Foundation
import CountriesManagerTypes
#if os(macOS)
import HelperInstall
#endif
import UIComponents

enum HomeLink: Hashable, Identifiable {
    case gatewayDetails(gateway: GatewayNode, hopType: HopType)
    case entryGateways
    case exitGateways
    case settings
#if os(macOS)
    case installHelper(afterInstallAction: HelperAfterInstallAction)
#endif

    var id: String {
        String(describing: self)
    }
}
