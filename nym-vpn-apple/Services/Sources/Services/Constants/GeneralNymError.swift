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
    case somethingWentWrong
}

extension GeneralNymError: LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .invalidUrl:
            "generalNymError.invalidUrl".localizedString
        case .cannotFetchCountries:
            "generalNymError.cannotFetchCountries".localizedString
        case .noPrebundledCountries:
            "generalNymError.noPrebundledCountries".localizedString
        case .cannotParseCountries:
            "generalNymError.cannotParseCountries".localizedString
        case .library(message: let message):
            message
        case .noMnemonicStored:
            "error.noMnemonicStored".localizedString
        case .noEnv:
            "generalNymError.noEnv".localizedString
        case .somethingWentWrong:
            "generalNymError.somethingWentWrong".localizedString
        }
    }
}
