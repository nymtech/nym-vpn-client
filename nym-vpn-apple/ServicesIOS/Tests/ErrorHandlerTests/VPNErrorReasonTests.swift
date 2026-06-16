import Testing
import NymVPNLib
@testable import ErrorHandler

struct VPNErrorReasonTests {
    @Test func accountStoreBusyMapsFromVpnError() {
        let reason = VPNErrorReason(with: VpnError.AccountStoreBusy)
        #expect(reason == .accountStoreBusy)
    }

    @Test func accountStoreBusyRoundTripsThroughNSError() {
        let original = VPNErrorReason.accountStoreBusy
        let restored = VPNErrorReason(nsError: original.nsError)
        #expect(restored == .accountStoreBusy)
    }

    @Test func accountStoreBusyHasNonEmptyDescription() {
        #expect(VPNErrorReason.accountStoreBusy.errorDescription?.isEmpty == false)
    }

    @Test func storageLockDetailMapsToAccountStoreBusy() {
        let reason = VPNErrorReason(with: VpnError.Storage(details: "failed to acquire credential store lock"))
        #expect(reason == .accountStoreBusy)
    }
}
