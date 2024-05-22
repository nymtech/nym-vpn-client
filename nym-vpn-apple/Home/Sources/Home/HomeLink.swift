import Foundation
import CountriesManager

enum HomeLink: Hashable, Identifiable {
    case entryHop
    case exitHop
    case settings

    var id: String {
        String(describing: self)
    }
}
