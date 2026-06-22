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
}
