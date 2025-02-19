import Foundation
import CountriesManager
#if os(macOS)
import HelperInstall
#endif

enum HomeLink: Hashable, Identifiable {
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
