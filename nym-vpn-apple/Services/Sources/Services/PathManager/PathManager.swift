import Foundation
import Constants

public final class PathManager {
    /// Group folder, created automatically if does not exists
    /// `/private/var/mobile/Containers/Shared/AppGroup/xxx-xxx-xxx-xxx-xxx/Data/`
    /// - Returns: URL to group data folder
    public nonisolated static func dataFolderURL() throws -> URL {
        guard let dataFolderURL = FileManager.default
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("Data")
        else {
            throw PathManagerError.cannotCreateDB
        }
        if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
            try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
        }
        return dataFolderURL
    }

    public nonisolated static func cacheFolderURL() throws -> URL {
        try Self.dataFolderURL().appendingPathComponent("Cache")
    }

    public nonisolated static func configFolderURL() throws -> URL {
        try Self.dataFolderURL().appendingPathComponent("Config")
    }
}
