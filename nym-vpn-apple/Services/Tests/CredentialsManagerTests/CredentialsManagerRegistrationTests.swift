#if os(iOS)
import Foundation
import Testing
@testable import CredentialsManager

@MainActor
struct CredentialsManagerRegistrationTests {
    @Test func beginRegistrationSetsInFlightFlag() {
        let manager = CredentialsManager.shared
        manager.endAccountRegistration()
        #expect(manager.isAccountRegistrationInFlight == false)

        manager.beginAccountRegistration()
        #expect(manager.isAccountRegistrationInFlight == true)

        manager.endAccountRegistration()
        #expect(manager.isAccountRegistrationInFlight == false)
    }
}
#endif
