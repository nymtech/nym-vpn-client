import Foundation
import Constants
import MixnetLibrary

public final class CredentialsManager {
    public static let shared = CredentialsManager()

    public func add(credential: String) throws {
        do {
            let dataFolderURL = try dataFolderURL()

            FileManager.default.createFile(atPath: dataFolderURL.path(), contents: nil)

            try importCredential(credential: credential, path: dataFolderURL.path())
        } catch let error {
            throw error
        }
    }

    public func dataFolderURL() throws -> URL {
        guard let dataFolderURL = FileManager.default
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("Data")
        else {
            throw CredentialsManagerError.cannotCreateDB
        }
        return dataFolderURL
    }
}
