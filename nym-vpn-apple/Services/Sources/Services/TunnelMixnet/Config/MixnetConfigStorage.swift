import Foundation
import Logging
import PathManager
import Security

public enum MixnetConfigStorage {
    private static let fileName = "MixnetConfig.json"
    private static let legacyKeychainLabel = "WireGuard Tunnel: NymVPN Mixnet"
    private static let logger = Logger(label: "MixnetConfigStorage")

    private static func fileURL() throws -> URL {
        let folder = try PathManager.configFolderURL()
        if !FileManager.default.fileExists(atPath: folder.path()) {
            try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
        }
        return folder.appendingPathComponent(fileName)
    }

    @discardableResult
    public static func save(_ config: MixnetConfig) -> Bool {
        guard let json = config.toJson() else { return false }
        do {
            let url = try fileURL()
            try json.write(to: url, atomically: true, encoding: .utf8)
            return true
        } catch {
            logger.error("Failed to write MixnetConfig: \(error.localizedDescription)")
            return false
        }
    }

    public static func load() -> MixnetConfig? {
        do {
            let url = try fileURL()
            guard FileManager.default.fileExists(atPath: url.path()) else { return nil }
            let json = try String(contentsOf: url, encoding: .utf8)
            return MixnetConfig.from(jsonString: json)
        } catch {
            logger.error("Failed to read MixnetConfig: \(error.localizedDescription)")
            return nil
        }
    }

    public static func delete() {
        do {
            let url = try fileURL()
            guard FileManager.default.fileExists(atPath: url.path()) else { return }
            try FileManager.default.removeItem(at: url)
        } catch {
            logger.error("Failed to delete MixnetConfig: \(error.localizedDescription)")
        }
    }

    public static func exists() -> Bool {
        guard let url = try? fileURL() else { return false }
        return FileManager.default.fileExists(atPath: url.path())
    }

    /// One-time migration: if no file exists but legacy keychain entry does, copy JSON to file
    /// and delete the keychain item. Safe to call repeatedly — no-op once migrated.
    @discardableResult
    public static func migrateFromKeychainIfNeeded() -> Bool {
        guard !exists() else { return false }

        guard var bundleIdentifier = Bundle.main.bundleIdentifier else { return false }
        if bundleIdentifier.hasSuffix(".network-extension") {
            bundleIdentifier.removeLast(".network-extension".count)
        }

        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: bundleIdentifier,
            kSecAttrLabel: legacyKeychainLabel,
            kSecMatchLimit: kSecMatchLimitOne,
            kSecReturnData: true,
            kSecReturnPersistentRef: true
        ]

        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess,
              let dict = result as? [CFString: Any],
              let data = dict[kSecValueData] as? Data,
              let ref = dict[kSecValuePersistentRef] as? Data,
              let json = String(data: data, encoding: .utf8)
        else {
            return false
        }

        do {
            let url = try fileURL()
            try json.write(to: url, atomically: true, encoding: .utf8)
        } catch {
            logger.error("Keychain migration: failed to write file: \(error.localizedDescription)")
            return false
        }

        let deleteStatus = SecItemDelete([kSecValuePersistentRef: ref] as CFDictionary)
        if deleteStatus != errSecSuccess {
            logger.warning("Keychain migration: file written but legacy entry delete returned \(deleteStatus)")
        }
        logger.info("Migrated MixnetConfig from keychain to app-group file")
        return true
    }
}
