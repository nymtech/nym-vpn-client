import Foundation
import Constants
#if os(iOS)
import MixnetLibrary

#endif
#if os(macOS)
import GRPCManager
#endif

public final class CredentialsManager {
    public static let shared = CredentialsManager()

    public func add(credential: String) throws {
        let trimmedCredential = credential.trimmingCharacters(in: .whitespacesAndNewlines)
        do {
#if os(iOS)
            let dataFolderURL = try dataFolderURL()

            if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
                try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
            }
            try importCredential(credential: trimmedCredential, path: dataFolderURL.path())
#endif
#if os(macOS)
            try GRPCManager.shared.importCredential(credential: trimmedCredential)
#endif
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
