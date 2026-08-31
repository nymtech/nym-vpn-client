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

    @Test func independenceConsentUsesAwaitingGatewayConsentArc() {
        #expect(
            GatewayIndependenceArcPolicy.shouldUseAwaitingGatewayConsentArc(
                status: .error,
                lastError: ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
        #expect(
            !GatewayIndependenceArcPolicy.shouldUseAwaitingGatewayConsentArc(
                status: .error,
                lastError: ErrorReason.internalUnknown
            )
        )
        #expect(
            !GatewayIndependenceArcPolicy.shouldUseAwaitingGatewayConsentArc(
                status: .connecting,
                lastError: ErrorReason.needsRelaxedIndependenceCriteria
            )
        )
    }

    @Test func appConnectAfterRelaxConsentOnlyWhenDisconnected() {
        #expect(
            GatewayIndependenceArcPolicy.shouldAppInitiateConnectAfterRelaxConsent(
                status: .disconnected
            )
        )
        #expect(
            !GatewayIndependenceArcPolicy.shouldAppInitiateConnectAfterRelaxConsent(
                status: .error
            )
        )
        #expect(
            !GatewayIndependenceArcPolicy.shouldAppInitiateConnectAfterRelaxConsent(
                status: .connecting
            )
        )
    }

    @Test func independenceConsentPreservesLastErrorOnErrorStatus() {
        let error = ErrorReason.needsRelaxedIndependenceCriteria
        #expect(
            GatewayIndependenceArcPolicy.shouldPreserveIndependenceConsentError(
                status: .error,
                lastError: error
            )
        )
        #expect(
            !GatewayIndependenceArcPolicy.shouldPreserveIndependenceConsentError(
                status: .error,
                lastError: ErrorReason.internalUnknown
            )
        )
        #expect(
            !GatewayIndependenceArcPolicy.shouldPreserveIndependenceConsentError(
                status: .disconnected,
                lastError: error
            )
        )
    }
}
