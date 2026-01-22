import Foundation
import Constants

@Observable
public final class DeeplinkManager {
    public init () {}

    public func handle(url: URL) {
        guard url.scheme == Constants.appUrlScheme.rawValue,
            let components = URLComponents(url: url, resolvingAgainstBaseURL: true)
        else {
            return
        }
    }
}
