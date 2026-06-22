import Foundation
import Testing
import NymVPNLib
@testable import ErrorHandler

struct VpnErrorUniFFIBoundaryTests {
    @Test func passthroughExistingVpnError() {
        let original = VpnError.NoAccountStored
        let mapped = VpnErrorUniFFIBoundary.vpnError(from: original)
        guard case .NoAccountStored = mapped else {
            Issue.record("Expected NoAccountStored passthrough")
            return
        }
    }

    @Test func mapsNEAgentErrorToInternalVpnError() {
        let neError = NSError(domain: "NEAgentErrorDomain", code: 1, userInfo: nil)
        let mapped = VpnErrorUniFFIBoundary.vpnError(from: neError)
        guard case let .InternalError(details: details) = mapped else {
            Issue.record("Expected InternalError for NEAgentErrorDomain")
            return
        }
        #expect(details.contains("NEAgentErrorDomain"))
        #expect(details.contains("1"))
    }
}
