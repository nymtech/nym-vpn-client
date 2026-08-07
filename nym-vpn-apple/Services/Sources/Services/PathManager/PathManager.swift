import Foundation
import Logging
import Constants

public final class PathManager {
    private static let logger = Logger(label: "PathManager")

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
        if !isExcludedFromBackup(dataFolderURL) {
            do {
                try excludeFromBackup(dataFolderURL)
            } catch {
                logger.error("Failed to exclude from backups with \(error.localizedDescription)")
            }
        }
        return dataFolderURL
    }

    public nonisolated static func configFolderURL() throws -> URL {
        try Self.dataFolderURL().appendingPathComponent("Config")
    }
}

private extension PathManager {
    static func isExcludedFromBackup(_ url: URL) -> Bool {
        let values = try? url.resourceValues(forKeys: [.isExcludedFromBackupKey])
        return values?.isExcludedFromBackup ?? false
    }

    static func excludeFromBackup(_ url: URL, isExcluded: Bool = true) throws {
        var resourceValues = URLResourceValues()
        resourceValues.isExcludedFromBackup = isExcluded
        var mutableURL = url
        try mutableURL.setResourceValues(resourceValues)
    }
}
