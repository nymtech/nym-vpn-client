import Foundation
import Testing
import AccountPrefetchGates

struct OnboardingLocalizationCatalogTests {
    private static func englishValue(for key: String) throws -> String {
        let url = try catalogURL()
        let data = try Data(contentsOf: url)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        let strings = json?["strings"] as? [String: Any]
        let entry = strings?[key] as? [String: Any]
        let localizations = entry?["localizations"] as? [String: Any]
        let en = localizations?["en"] as? [String: Any]
        let unit = en?["stringUnit"] as? [String: Any]
        guard let value = unit?["value"] as? String else {
            throw CatalogError.missingKey(key)
        }
        return value
    }

    private static func catalogURL() throws -> URL {
        if let override = ProcessInfo.processInfo.environment["XCSTRINGS_PATH"], !override.isEmpty {
            return URL(fileURLWithPath: override)
        }
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<3 { url.deleteLastPathComponent() }
        return url
            .appendingPathComponent("NymVPN")
            .appendingPathComponent("Resources")
            .appendingPathComponent("Localizable.xcstrings")
    }

    @Test func postPurchaseProcessingKeysHaveEnglishCopy() throws {
        let title = try englishValue(for: PostPurchaseProcessingUI.titleKey)
        let subtitle = try englishValue(for: PostPurchaseProcessingUI.subtitleKey)
        #expect(title != PostPurchaseProcessingUI.titleKey)
        #expect(subtitle != PostPurchaseProcessingUI.subtitleKey)
        #expect(!title.isEmpty)
        #expect(!subtitle.isEmpty)
    }

    @Test func loginProcessingKeysHaveEnglishCopy() throws {
        let title = try englishValue(for: LoginProcessingUI.titleKey)
        let subtitle = try englishValue(for: LoginProcessingUI.subtitleKey)
        #expect(title != LoginProcessingUI.titleKey)
        #expect(subtitle != LoginProcessingUI.subtitleKey)
    }
}

private enum CatalogError: Error {
    case missingKey(String)
}
