import Testing
import AccountPrefetchGates

struct SantaStoreKitEnvironmentPolicyTests {
    @Test func testFlightGuidanceMentionsTestFlightAppleID() {
        let message = SantaStoreKitEnvironmentPolicy.guidanceMessage(isTestFlight: true)
        #expect(message.contains("TestFlight Apple ID"))
        #expect(message.contains("Developer Settings sandbox account"))
    }

    @Test func xcodeDebugGuidanceMentionsDeveloperSandboxAccount() {
        let message = SantaStoreKitEnvironmentPolicy.guidanceMessage(isTestFlight: false)
        #expect(message.contains("Sandbox Apple Account"))
        #expect(message.contains("does not change the StoreKit Apple ID"))
    }
}
