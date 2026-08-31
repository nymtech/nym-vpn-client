#if os(macOS)
import Foundation
import Testing
@testable import CredentialsManager

@MainActor
struct CredentialsManagerMacOSRegistrationTests {
    @Test func registerAccountIsNoOpOnMacOS() async throws {
        try await CredentialsManager.shared.registerAccount()
    }
}
#endif
