import Foundation

/// Reads the app's `Localizable.xcstrings` from disk and resolves the English value for a key.
struct XCStringsResolver {
    private let strings: [String: Any]

    static func defaultURL(file: StaticString = #filePath) -> URL {
        if let override = ProcessInfo.processInfo.environment["XCSTRINGS_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        var url = URL(fileURLWithPath: "\(file)")
        for _ in 0..<4 { url.deleteLastPathComponent() }
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
