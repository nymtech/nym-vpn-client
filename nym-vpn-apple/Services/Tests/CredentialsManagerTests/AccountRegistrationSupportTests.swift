#if os(iOS)
import Foundation
import Testing
import ErrorHandler
import NymVPNLib
@testable import CredentialsManager

struct AccountRegistrationSupportTests {
    @Test func detectsAccountStoreBusyVpnError() {
        #expect(AccountRegistrationSupport.isCredentialStoreLockFailure(VpnError.AccountStoreBusy))
    }

    @Test func detectsStorageLockDetailVpnError() {
        let error = VpnError.Storage(details: "failed to acquire credential store lock")
        #expect(AccountRegistrationSupport.isCredentialStoreLockFailure(error))
    }

    @Test func detectsAccountStoreBusyReason() {
        #expect(AccountRegistrationSupport.isCredentialStoreLockFailure(VPNErrorReason.accountStoreBusy))
    }

    @Test func detectsStorageLockDetailReason() {
        let error = VPNErrorReason.storage(details: "failed to acquire credential store lock")
        #expect(AccountRegistrationSupport.isCredentialStoreLockFailure(error))
    }

    @Test func ignoresUnrelatedStorageError() {
        let error = VPNErrorReason.storage(details: "disk full")
        #expect(!AccountRegistrationSupport.isCredentialStoreLockFailure(error))
    }

    @Test func mapsVpnStorageLockToReasonDescription() {
        let vpnError = VpnError.Storage(details: "failed to acquire credential store lock")
        let mapped = AccountRegistrationSupport.mapToVPNErrorReason(vpnError) as? VPNErrorReason
        #expect(mapped != nil)
        if case let .storage(details) = mapped {
            #expect(details.contains("credential store lock"))
        } else {
            Issue.record("expected storage reason")
        }
    }
}
#endif
