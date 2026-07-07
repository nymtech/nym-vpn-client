#if os(iOS)
import Testing
@testable import CredentialsManager

@MainActor
struct CredentialsManagerLogoutTests {
    @Test func beginLogoutKeepsGuardUntilEndLogout() async {
        let manager = CredentialsManager.shared
        manager.endLogout()
        #expect(manager.isLoggingOut == false)

        await manager.beginLogout()
        #expect(manager.isLoggingOut == true)

        manager.endLogout()
        #expect(manager.isLoggingOut == false)
    }

    @Test func updateAccountSummaryNoOpsWhileLoggingOut() async {
        let manager = CredentialsManager.shared
        manager.endLogout()
        await manager.beginLogout()
        defer { manager.endLogout() }

        let summaryBefore = manager.accountSummary
        await manager.updateAccountSummary(force: true)
        #expect(manager.accountSummary == summaryBefore)
    }
}
#endif
