import Foundation
import Testing
import AccountPrefetchGates
import Theme

struct OnboardingLocalizationCatalogTests {
    private static let processingKeys: [String] = [
        PostPurchaseProcessingUI.titleKey,
        PostPurchaseProcessingUI.subtitleKey
    ] + LoginProcessingUI.carouselKeys

    @Test func postPurchaseProcessingKeysHaveEnglishCopy() throws {
        let title = try NymVPNXCStringsReader.englishValue(for: PostPurchaseProcessingUI.titleKey)
        let subtitle = try NymVPNXCStringsReader.englishValue(for: PostPurchaseProcessingUI.subtitleKey)
        #expect(title != PostPurchaseProcessingUI.titleKey)
        #expect(subtitle != PostPurchaseProcessingUI.subtitleKey)
        #expect(!title.isEmpty)
        #expect(!subtitle.isEmpty)
    }

    @Test func loginProcessingKeysHaveEnglishCopy() throws {
        for key in LoginProcessingUI.carouselKeys {
            let value = try NymVPNXCStringsReader.englishValue(for: key)
            #expect(value != key)
            #expect(!value.isEmpty)
        }
    }

    @Test func processingKeysMarkedTranslatedInCatalog() throws {
        for key in Self.processingKeys {
            let state = try NymVPNXCStringsReader.englishTranslationState(for: key)
            #expect(state == "translated", "Expected translated state for \(key), got \(state)")
        }
    }

    @Test func processingKeysResolveThroughLocalizedStringPipeline() throws {
        for key in Self.processingKeys {
            let catalogEnglish = try NymVPNXCStringsReader.englishValue(for: key)
            #expect(catalogEnglish != key)

            let resolved = key.localizedString
            if resolved != key {
                #expect(resolved == catalogEnglish)
            } else if let appBundle = Self.nymVPNAppBundleFromEnvironment() {
                let inApp = Self.resolve(key: key, bundle: appBundle)
                #expect(inApp != key)
                #expect(inApp == catalogEnglish)
            }
        }
    }

    @Test func localizedStringPipelineResolvesEnglishStringsTable() throws {
        let key = LoginProcessingUI.carouselKeys[0]
        let expected = try NymVPNXCStringsReader.englishValue(for: key)
        let bundle = try Self.temporaryEnglishBundle(
            entries: [key: expected]
        )
        let resolved = Self.resolve(key: key, bundle: bundle)
        #expect(resolved != key)
        #expect(resolved == expected)
    }

    private static func resolve(key: String, bundle: Bundle) -> String {
        let catalog = String(localized: String.LocalizationValue(key), bundle: bundle)
        if catalog != key {
            return catalog
        }
        return bundle.localizedStringFallback(forKey: key)
    }

    private static func nymVPNAppBundleFromEnvironment() -> Bundle? {
        guard let path = ProcessInfo.processInfo.environment["NYMVPN_APP_BUNDLE_PATH"],
              !path.isEmpty else {
            return nil
        }
        return Bundle(path: path)
    }

    private static func temporaryEnglishBundle(entries: [String: String]) throws -> Bundle {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("nymvpn-l10n-test-\(UUID().uuidString)", isDirectory: true)
        let enDir = root.appendingPathComponent("en.lproj", isDirectory: true)
        try FileManager.default.createDirectory(at: enDir, withIntermediateDirectories: true)

        let body = entries
            .map { "\"\($0.key)\" = \"\($0.value.replacingOccurrences(of: "\"", with: "\\\""))\";" }
            .joined(separator: "\n")
        try body.write(
            to: enDir.appendingPathComponent("Localizable.strings"),
            atomically: true,
            encoding: .utf8
        )
        guard let bundle = Bundle(path: root.path) else {
            throw NymVPNXCStringsReader.ReadError.missingKey("bundle")
        }
        return bundle
    }
}
