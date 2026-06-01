import Foundation

/// Reads the app's `Localizable.xcstrings` from disk and resolves the English
/// (source-language) value for a key. Used because `.localizedString` reads
/// `Bundle.main`, which under the package test bundle has no app strings.
struct XCStringsResolver {
    private let strings: [String: Any]

    /// Locates the xcstrings: env override `XCSTRINGS_PATH`, else a path relative
    /// to this source file (…/ServicesMutual/Tests/ConnectionTypesTests → nym-vpn-apple/).
    static func defaultURL(file: StaticString = #filePath) -> URL {
        if let override = ProcessInfo.processInfo.environment["XCSTRINGS_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        // file = .../nym-vpn-apple/ServicesMutual/Tests/ConnectionTypesTests/XCStringsResolver.swift
        var url = URL(fileURLWithPath: "\(file)")
        for _ in 0..<4 { url.deleteLastPathComponent() } // → nym-vpn-apple/
        return url
            .appendingPathComponent("NymVPN")
            .appendingPathComponent("Resources")
            .appendingPathComponent("Localizable.xcstrings")
    }

    static func `default`() throws -> XCStringsResolver {
        try XCStringsResolver(url: defaultURL())
    }

    init(url: URL) throws {
        let data = try Data(contentsOf: url)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        strings = (json?["strings"] as? [String: Any]) ?? [:]
    }

    /// English value for `key`, or `key` itself when missing.
    func string(_ key: String) -> String {
        guard
            let entry = strings[key] as? [String: Any],
            let locs = entry["localizations"] as? [String: Any],
            let en = locs["en"] as? [String: Any],
            let unit = en["stringUnit"] as? [String: Any],
            let value = unit["value"] as? String
        else { return key }
        return value
    }
}
