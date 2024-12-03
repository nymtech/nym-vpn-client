import Foundation
import Theme

public enum DaemonError: Error, Equatable {
    case authorizationDenied
}

extension DaemonError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .authorizationDenied:
            "daemonError.authorizationDenied".localizedString
        }
    }
}
