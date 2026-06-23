import Foundation
import ErrorReason

/// Arc presentation for gateway independence consent. Keys only on
/// `ErrorReason.needsRelaxedIndependenceCriteria` from Rust (any VPN environment).
public enum GatewayIndependenceArcPolicy {
    public static func isIndependenceConsentError(_ error: Error?) -> Bool {
        guard let error else { return false }
        let reason = (error as? ErrorReason) ?? ErrorReason(nsError: error as NSError)
        return reason == .needsRelaxedIndependenceCriteria
    }

    /// True when the connection arc should use the red failed state for a tunnel error.
    public static func shouldUseFailedArc(status: TunnelStatus, lastError: Error?) -> Bool {
        guard status == .error else { return false }
        return !isIndependenceConsentError(lastError)
    }

    /// True when a tunnel error should stick as a connection failure after disconnect.
    public static func shouldRecordConnectionFailure(_ error: Error?) -> Bool {
        !isIndependenceConsentError(error)
    }

    /// True when the arc should show the gateway-independence consent state instead of a connect step.
    public static func shouldUseAwaitingGatewayConsentArc(status: TunnelStatus, lastError: Error?) -> Bool {
        status == .error && isIndependenceConsentError(lastError)
    }

    /// Whether the iOS app should call `connect` after `setGatewayIndependence(false)`.
    /// The NE extension resumes pre-flight or calls `connectTunnel` for every other
    /// status (including `.error`); an app-side reconnect races that handler.
    public static func shouldAppInitiateConnectAfterRelaxConsent(status: TunnelStatus) -> Bool {
        status == .disconnected
    }

    /// Independence consent must keep `lastError` while the tunnel stays on `.error`.
    public static func shouldPreserveIndependenceConsentError(
        status: TunnelStatus,
        lastError: Error?
    ) -> Bool {
        status == .error && isIndependenceConsentError(lastError)
    }
}
