import Testing
import ErrorReason
import TunnelStatus

struct GatewayIndependenceArcPolicyTests {
    @Test func independenceConsentErrorRecognizesErrorReason() {
        #expect(
            GatewayIndependenceArcPolicy.isIndependenceConsentError(
                ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
    }

    @Test func independenceConsentErrorRecognizesNSErrorFromTunnel() {
        let nsError = ErrorReason.needsRelaxedIndependenceCriteria.nsError
        #expect(GatewayIndependenceArcPolicy.isIndependenceConsentError(nsError))
    }

    @Test func independenceConsentErrorRejectsGenericFailure() {
        #expect(
            !GatewayIndependenceArcPolicy.isIndependenceConsentError(
                ErrorReason.internalUnknown
            )
        )
        #expect(!GatewayIndependenceArcPolicy.isIndependenceConsentError(nil))
    }

    @Test func errorStatusWithIndependenceDoesNotUseFailedArc() {
        #expect(
            !GatewayIndependenceArcPolicy.shouldUseFailedArc(
                status: .error,
                lastError: ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
    }

    @Test func errorStatusWithGenericErrorUsesFailedArc() {
        #expect(
            GatewayIndependenceArcPolicy.shouldUseFailedArc(
                status: .error,
                lastError: ErrorReason.internalUnknown
            )
        )
    }

    @Test func independenceConsentDoesNotRecordConnectionFailure() {
        #expect(
            !GatewayIndependenceArcPolicy.shouldRecordConnectionFailure(
                ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
        #expect(
            GatewayIndependenceArcPolicy.shouldRecordConnectionFailure(
                ErrorReason.internalUnknown
            )
        )
    }
}
