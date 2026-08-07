// swiftlint:disable all

import Foundation
import Security
import Constants

@MainActor
public class Keychain {
    public static func addInternetPassword(with password: String) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            let domain = Constants.domainName.rawValue
            let account = Constants.username.rawValue
            SecAddSharedWebCredential(
                domain as CFString,
                account as CFString,
                password as CFString
            ) { error in
                if let error = error as? NSError {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume()
                }
            }
        }
    }
}
// swiftlint:enable all
