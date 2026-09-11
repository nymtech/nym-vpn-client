import Testing
import TunnelStatus

struct AccountControllerLoginStateTests {
    @Test func inactiveKindsAreTerminalForLogin() {
        #expect(AccountControllerLoginState.inactiveSubscription.isTerminalInactiveForLogin)
        #expect(AccountControllerLoginState.accountStatusNotActive.isTerminalInactiveForLogin)
        #expect(!AccountControllerLoginState.other.isTerminalInactiveForLogin)
    }
}
