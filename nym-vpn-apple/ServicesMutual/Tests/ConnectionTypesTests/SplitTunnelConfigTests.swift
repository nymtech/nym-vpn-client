import Foundation
import Testing
@testable import ConnectionTypes

struct SplitTunnelConfigTests {
    private let decoder = JSONDecoder()
    private let encoder = JSONEncoder()

    @Test func decodesLegacyJSONWithoutCustomAppPaths() throws {
        let json = #"{"isEnabled":true,"appPaths":["/Applications/Foo.app/Contents/MacOS/Foo"]}"#
        let config = try decoder.decode(SplitTunnelConfig.self, from: Data(json.utf8))

        #expect(config.isEnabled)
        #expect(config.appPaths == ["/Applications/Foo.app/Contents/MacOS/Foo"])
        #expect(config.customAppPaths.isEmpty)
    }

    @Test func decodesJSONWithCustomAppPaths() throws {
        let json = #"{"isEnabled":false,"appPaths":[],"customAppPaths":["/Users/me/Applications/Bar.app/Contents/MacOS/Bar"]}"#
        let config = try decoder.decode(SplitTunnelConfig.self, from: Data(json.utf8))

        #expect(config.customAppPaths == ["/Users/me/Applications/Bar.app/Contents/MacOS/Bar"])
    }

    @Test func decodesExplicitNullCustomAppPathsAsEmpty() throws {
        let json = #"{"isEnabled":false,"appPaths":[],"customAppPaths":null}"#
        let config = try decoder.decode(SplitTunnelConfig.self, from: Data(json.utf8))

        #expect(config.customAppPaths.isEmpty)
    }

    @Test func encodeDecodeRoundTripPreservesCustomAppPaths() throws {
        let original = SplitTunnelConfig(
            isEnabled: true,
            appPaths: ["/a/X.app/Contents/MacOS/X"],
            customAppPaths: ["/b/Y.app/Contents/MacOS/Y"]
        )
        let data = try encoder.encode(original)
        let restored = try decoder.decode(SplitTunnelConfig.self, from: data)

        #expect(restored == original)
    }
}
