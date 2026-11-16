import Testing
@testable import ConnectionTypes

@Suite struct TransportPreferenceSettingTests {
    @Test(arguments: TransportPreferenceSetting.allCases)
    func roundTripsThroughBridgeTransport(setting: TransportPreferenceSetting) {
        #expect(TransportPreferenceSetting(setting.bridgeTransport) == setting)
    }

    @Test func autoMapsToNoTransportPreference() {
        #expect(TransportPreferenceSetting.auto.bridgeTransport == nil)
        #expect(TransportPreferenceSetting(nil) == .auto)
    }
}
