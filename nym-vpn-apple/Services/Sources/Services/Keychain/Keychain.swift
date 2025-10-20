// swiftlint:disable all

import Foundation
import Security
import Constants
import Logging

@MainActor
public class Keychain {
    private static var logger = Logger(label: "KeychainLogger")

    public static func openReference(called ref: Data) -> String? {
        var result: CFTypeRef?
        let ret = SecItemCopyMatching(
            [
                kSecValuePersistentRef: ref,
                kSecReturnData: true
            ] as CFDictionary,
            &result
        )
        if ret != errSecSuccess || result == nil {
            // TODO: logger
            return nil
        }
        guard let data = result as? Data else { return nil }
        return String(data: data, encoding: String.Encoding.utf8)
    }

    public static func makeReference(
        containing value: String,
        called name: String,
        previouslyReferencedBy oldRef: Data? = nil
    ) -> Data? {
        var ret: OSStatus
        guard var bundleIdentifier = Bundle.main.bundleIdentifier else {
            // TODO: logger
            return nil
        }
        if bundleIdentifier.hasSuffix(".network-extension") {
            bundleIdentifier.removeLast(".network-extension".count)
        }
        let itemLabel = "WireGuard Tunnel: \(name)"
        var items: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrLabel: itemLabel,
            kSecAttrAccount: name + ": " + UUID().uuidString,
            kSecAttrDescription: "wg-quick(8) config",
            kSecAttrService: bundleIdentifier,
            kSecValueData: value.data(using: .utf8) as Any,
            kSecReturnPersistentRef: true
        ]

        #if os(iOS)
        items[kSecAttrAccessGroup] = Constants.groupID.rawValue
        items[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
        #elseif os(macOS)
        items[kSecAttrSynchronizable] = false
        items[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly

        let pathComponent = "NymVPN.appex"
        guard let extensionPath = Bundle.main.builtInPlugInsURL?.appendingPathComponent(pathComponent, isDirectory: true).path
        else {
            logger.log(level: .error, "Unable to determine app extension path")
            return nil
        }
        var extensionApp: SecTrustedApplication?
        var mainApp: SecTrustedApplication?
        ret = SecTrustedApplicationCreateFromPath(extensionPath, &extensionApp)
        if ret != kOSReturnSuccess || extensionApp == nil {
            logger.log(level: .error, "Unable to create keychain extension trusted application object: \(ret)")
            return nil
        }
        ret = SecTrustedApplicationCreateFromPath(nil, &mainApp)
        if ret != errSecSuccess || mainApp == nil {
            logger.log(level: .error, "Unable to create keychain local trusted application object: \(ret)")
            return nil
        }
        var access: SecAccess?
        ret = SecAccessCreate(itemLabel as CFString, [extensionApp!, mainApp!] as CFArray, &access)
        if ret != errSecSuccess || access == nil {
            logger.log(level: .error, "Unable to create keychain ACL object: \(ret)")
            return nil
        }
        items[kSecAttrAccess] = access!
        #else
        #error("Unimplemented")
        #endif

        var ref: CFTypeRef?
        ret = SecItemAdd(items as CFDictionary, &ref)
        if ret != errSecSuccess || ref == nil {
            // TODO: logger
            return nil
        }
        if let oldRef = oldRef {
            deleteReference(called: oldRef)
        }
        return ref as? Data
    }

    public static func updateReferenceOrCreateNew(called name: String, with value: String) -> Data? {
        guard var bundleIdentifier = Bundle.main.bundleIdentifier
        else {
            logger.error("Missing bundle identifier")
            return nil
        }
        if bundleIdentifier.hasSuffix(".network-extension") {
            bundleIdentifier.removeLast(".network-extension".count)
        }
        let itemLabel = "WireGuard Tunnel: \(name)"

        var result: CFTypeRef?
        let findQuery: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: bundleIdentifier,
            kSecAttrLabel: itemLabel,
            kSecMatchLimit: kSecMatchLimitOne,
            kSecReturnPersistentRef: true
        ]
        let copyStatus = SecItemCopyMatching(findQuery as CFDictionary, &result)

        if copyStatus == errSecSuccess, let reference = result as? Data {
            let updateQuery: [CFString: Any] = [kSecValuePersistentRef: reference]
            let attributes: [CFString: Any] = [kSecValueData: value.data(using: .utf8) as Any]

            let updateStatus = SecItemUpdate(updateQuery as CFDictionary, attributes as CFDictionary)
            if updateStatus == errSecSuccess {
                return reference
            }
        }
        return makeReference(containing: value, called: name, previouslyReferencedBy: nil)
    }

    public static func deleteReference(called ref: Data) {
        let ret = SecItemDelete([kSecValuePersistentRef: ref] as CFDictionary)
        if ret != errSecSuccess {
            // TODO: logger
        }
    }

    public static func deleteReferences(except whitelist: Set<Data>) {
        var result: CFTypeRef?
        let ret = SecItemCopyMatching(
            [
                kSecClass: kSecClassGenericPassword,
                kSecAttrService: Bundle.main.bundleIdentifier as Any,
                kSecMatchLimit: kSecMatchLimitAll,
                kSecReturnPersistentRef: true
            ] as CFDictionary,
            &result
        )
        if ret != errSecSuccess || result == nil {
            return
        }
        guard let items = result as? [Data] else { return }
        for item in items {
            if !whitelist.contains(item) {
                deleteReference(called: item)
            }
        }
    }

    public static func verifyReference(called ref: Data) -> Bool {
        return SecItemCopyMatching([kSecValuePersistentRef: ref] as CFDictionary, nil) != errSecItemNotFound
    }


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
