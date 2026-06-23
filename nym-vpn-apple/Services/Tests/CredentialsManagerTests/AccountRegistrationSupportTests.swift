#if os(iOS)
import Foundation
import Testing
import ErrorHandler
import NymVPNLib
@testable import CredentialsManager

struct AccountRegistrationSupportTests {
    @Test func detectsAccountStoreBusyReason() {
        #expect(AccountRegistrationSupport.isAccountStoreBusyFailure(VPNErrorReason.accountStoreBusy))
    }

    @Test func ignoresUnrelatedStorageError() {
        let error = VPNErrorReason.storage(details: "disk full")
        #expect(!AccountRegistrationSupport.isAccountStoreBusyFailure(error))
    }

    @Test func ignoresUnrelatedVpnStorageError() {
        let error = VpnError.Storage(details: "disk full")
        #expect(!AccountRegistrationSupport.isAccountStoreBusyFailure(error))
    }

    @Test func mapsVpnStorageErrorToStorageReason() {
        let vpnError = VpnError.Storage(details: "disk full")
        let mapped = VPNErrorReason(with: vpnError)
        #expect(mapped == .storage(details: "disk full"))
    }

    @Test func usesCapturedEnvironmentDuringRegistration() throws {
        let captured = try NymEnvironment.newWithMainnetFallback()
        let resolved = AccountRegistrationSupport.environmentForCredentialImport(
            isRegistrationInFlight: true,
            registrationCapturedEnvironment: captured,
            liveNetworkEnv: nil
        )
        #expect(resolved != nil)
    }

    @Test func usesLiveEnvironmentWhenNotRegistering() throws {
        let live = try NymEnvironment.newWithMainnetFallback()
        let resolved = AccountRegistrationSupport.environmentForCredentialImport(
            isRegistrationInFlight: false,
            registrationCapturedEnvironment: nil,
            liveNetworkEnv: live
        )
        #expect(resolved != nil)
    }
}
#endif
