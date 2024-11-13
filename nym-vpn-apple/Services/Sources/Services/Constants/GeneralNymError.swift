import Foundation
import Theme

public enum GeneralNymError: Error, Equatable {
    case invalidUrl
    case cannotFetchCountries
    case noPrebundledCountries
    case cannotParseCountries
    case library(message: String)
    case noMnemonicStored
    case noEnv
}

extension GeneralNymError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidUrl:
            return "generalNymError.invalidUrl".localizedString
        case .cannotFetchCountries:
            return "generalNymError.cannotFetchCountries".localizedString
        case .noPrebundledCountries:
            return "generalNymError.noPrebundledCountries".localizedString
        case .cannotParseCountries:
            return "generalNymError.cannotParseCountries".localizedString
        case .library(message: let message):
            return message
        case .noMnemonicStored:
            return "error.noMnemonicStored".localizedString
        case .noEnv:
            return "generalNymError.noEnv".localizedString
        }
    }
}
