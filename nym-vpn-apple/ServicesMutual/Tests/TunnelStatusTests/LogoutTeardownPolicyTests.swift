import Testing
import TunnelStatus

struct LogoutTeardownPolicyTests {
    @Test func disconnectedSkipsDisconnectWait() {
        #expect(!LogoutTeardownPolicy.needsDisconnectWait(for: .disconnected))
        #expect(!LogoutTeardownPolicy.shouldInitiateDisconnect(for: .disconnected))
    }

    @Test func disconnectingWaitsWithoutSecondDisconnect() {
        #expect(LogoutTeardownPolicy.needsDisconnectWait(for: .disconnecting))
        #expect(!LogoutTeardownPolicy.shouldInitiateDisconnect(for: .disconnecting))
    }

    @Test func connectedInitiatesDisconnectAndWaits() {
        #expect(LogoutTeardownPolicy.needsDisconnectWait(for: .connected))
        #expect(LogoutTeardownPolicy.shouldInitiateDisconnect(for: .connected))
    }

    @Test func disconnectWaitCapMatchesRustDisconnectBudget() {
        #expect(LogoutTeardownPolicy.disconnectWaitCapSeconds >= 5)
        #expect(LogoutTeardownPolicy.disconnectWaitCapSeconds <= 10)
    }
}
