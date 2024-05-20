#if os(iOS)
import CountriesManager
import MixnetLibrary

extension ExitRouter {
    var exitPoint: ExitPoint {
        switch self {
        case .country(let code):
            return .location(location: code)
        case .random:
            let randomCountry = CountriesManager.shared.exitCountries?.first
            // TODO: prebundle countries
            return .location(location: randomCountry?.code ?? "DE")
        }
    }
}
#endif
