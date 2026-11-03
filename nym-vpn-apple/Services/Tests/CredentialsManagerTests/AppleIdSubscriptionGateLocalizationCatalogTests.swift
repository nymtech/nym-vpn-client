import Foundation
import Testing
import AccountPrefetchGates

struct AppleIdSubscriptionGateLocalizationCatalogTests {
    @Test func keysHaveTranslatedEnglishCopy() throws {
        for key in AppleIdSubscriptionGateUI.allKeys {
            let value = try NymVPNXCStringsReader.englishValue(for: key)
            #expect(value != key)
            #expect(!value.isEmpty)
            #expect(try NymVPNXCStringsReader.englishTranslationState(for: key) == "translated")
        }
    }

    @Test func bodyKeysCarryOneDatePlaceholder() throws {
        for key in [AppleIdSubscriptionGateUI.bodyAutoRenewOnKey, AppleIdSubscriptionGateUI.bodyAutoRenewOffKey] {
            let value = try NymVPNXCStringsReader.englishValue(for: key)
            #expect(value.components(separatedBy: "%@").count == 2)
        }
    }
}
