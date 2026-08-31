import Foundation

/// Reads NymVPN `Localizable.xcstrings` for unit tests (SPM test bundle has no app strings).
enum NymVPNXCStringsReader {
    static func catalogURL(file: StaticString = #filePath) -> URL {
        if let override = ProcessInfo.processInfo.environment["XCSTRINGS_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        var url = URL(fileURLWithPath: "\(file)")
        for _ in 0..<3 { url.deleteLastPathComponent() }
        return url
            .appendingPathComponent("NymVPN")
            .appendingPathComponent("Resources")
            .appendingPathComponent("Localizable.xcstrings")
    }

    static func englishValue(for key: String, file: StaticString = #filePath) throws -> String {
        let data = try Data(contentsOf: catalogURL(file: file))
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        let strings = json?["strings"] as? [String: Any]
        let entry = strings?[key] as? [String: Any]
        let localizations = entry?["localizations"] as? [String: Any]
        let en = localizations?["en"] as? [String: Any]
        let unit = en?["stringUnit"] as? [String: Any]
        guard let value = unit?["value"] as? String else {
            throw ReadError.missingKey(key)
        }
        return value
    }

    static func englishTranslationState(for key: String, file: StaticString = #filePath) throws -> String {
        let data = try Data(contentsOf: catalogURL(file: file))
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        let strings = json?["strings"] as? [String: Any]
        let entry = strings?[key] as? [String: Any]
        let localizations = entry?["localizations"] as? [String: Any]
        let en = localizations?["en"] as? [String: Any]
        let unit = en?["stringUnit"] as? [String: Any]
        guard let state = unit?["state"] as? String else {
            throw ReadError.missingKey(key)
        }
        return state
    }

    enum ReadError: Error, CustomStringConvertible {
        case missingKey(String)

        var description: String {
            switch self {
            case .missingKey(let key):
                return "Missing English entry for key: \(key)"
            }
        }
    }
}
