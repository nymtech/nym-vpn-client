#if os(macOS)
import ConnectionTypes
import Foundation
import Testing
@testable import TunnelMixnet

@Suite struct MixnetConfigTests {
    private func json(bridge: [String: Any]) throws -> String {
        let encoder = JSONEncoder()
        var object: [String: Any] = [
            "entryGateway": try JSONSerialization.jsonObject(with: encoder.encode(EntryGateway.random)),
            "exitRouter": try JSONSerialization.jsonObject(with: encoder.encode(ExitRouter.random)),
            "mixnetTuning": ["backgroundTraffic": 0, "continuousTraffic": 1, "disablePoissonRate": false],
            "isTwoHopEnabled": true,
            "isStealthApiEnabled": false,
            "isAdBlockingEnabled": false,
            "isErrorReportingEnabled": false,
            "isStatisticsEnabled": false,
            "isLanBypassEnabled": false,
            "name": "NymVPN"
        ]
        object.merge(bridge) { _, new in new }
        return String(decoding: try JSONSerialization.data(withJSONObject: object), as: UTF8.self)
    }

    @Test(arguments: [(true, BridgeModeSetting.on), (false, BridgeModeSetting.off)])
    func legacyQuicFlagBecomesBridgeMode(legacyQuic: Bool, expected: BridgeModeSetting) throws {
        let config = try #require(MixnetConfig.from(jsonString: json(bridge: ["isQuicEnabled": legacyQuic])))
        #expect(config.bridgeMode == expected)
    }

    @Test func bridgeModeWinsOverLegacyQuicFlag() throws {
        let config = try #require(
            MixnetConfig.from(jsonString: json(bridge: ["bridgeMode": "auto", "isQuicEnabled": false]))
        )
        #expect(config.bridgeMode == .auto)
    }

    @Test func missingBridgeSettingFailsToDecode() throws {
        #expect(try MixnetConfig.from(jsonString: json(bridge: [:])) == nil)
    }
}
#endif
