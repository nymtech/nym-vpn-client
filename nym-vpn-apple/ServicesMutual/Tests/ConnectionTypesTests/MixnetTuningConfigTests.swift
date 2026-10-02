import Foundation
import Testing
import NymVPNLib
@testable import ConnectionTypes

@Suite struct MixnetTuningConfigTests {
    @Test func missingCoverKeyStaysOn() throws {
        let json = """
        {"backgroundTraffic":0,"continuousTraffic":1,"disablePoissonRate":false}
        """
        let decoded = try JSONDecoder().decode(MixnetTuningConfig.self, from: Data(json.utf8))
        #expect(decoded.disableBackgroundCoverTraffic == false)
        #expect(decoded.mixnetTrafficConfig().disableBackgroundCoverTraffic == false)
    }

    @Test func coverDisableRoundTripsToDaemon() throws {
        let config = MixnetTuningConfig(
            backgroundTraffic: .ms40,
            continuousTraffic: .ms20,
            dissablePoissonRate: false,
            disableBackgroundCoverTraffic: true
        )
        let data = try JSONEncoder().encode(config)
        let decoded = try JSONDecoder().decode(MixnetTuningConfig.self, from: data)
        let fromDaemon = MixnetTuningConfig(from: decoded.mixnetTrafficConfig())
        #expect(decoded.disableBackgroundCoverTraffic)
        #expect(fromDaemon.disableBackgroundCoverTraffic)
        #expect(fromDaemon.mixnetTrafficConfig().disableBackgroundCoverTraffic)
    }
}
