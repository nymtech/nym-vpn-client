import Testing
import NymVPNLib
@testable import ErrorHandler

struct VPNErrorReasonTests {
    @Test func accountStoreBusyRoundTripsThroughNSError() {
        let original = VPNErrorReason.accountStoreBusy
        let restored = VPNErrorReason(nsError: original.nsError)
        #expect(restored == .accountStoreBusy)
    }

    @Test func accountStoreBusyHasNonEmptyDescription() {
        #expect(VPNErrorReason.accountStoreBusy.errorDescription?.isEmpty == false)
    }

    @Test func storageDetailMapsToStorageReason() {
        let reason = VPNErrorReason(with: VpnError.Storage(details: "disk full"))
        #expect(reason == .storage(details: "disk full"))
    }
}
