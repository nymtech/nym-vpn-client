import Foundation
import ErrorReason

/// In-tunnel gateway independence handling (Tauri `useGatewayIndependenceWatcher` parity).
public enum GatewayIndependenceResponsePolicy {
    public enum Action: Equatable {
        case noAction
        case showModal
        case autoRelaxAndReconnect
    }

    /// What the app should do when the tunnel surfaces `NeedsRelaxedIndependenceCriteria`.
    public static func action(
        status: TunnelStatus,
        lastError: Error?,
        notificationsEnabled: Bool,
        isHandlingEpisode: Bool
    ) -> Action {
        guard GatewayIndependenceArcPolicy.isIndependenceConsentError(lastError) else {
            return .noAction
        }
        guard status == .error else { return .noAction }
        guard !isHandlingEpisode else { return .noAction }
        if notificationsEnabled {
            return .showModal
        }
        return .autoRelaxAndReconnect
    }

    public static func shouldClearHandlingEpisode(
        status: TunnelStatus,
        lastError: Error?
    ) -> Bool {
        status != .error || !GatewayIndependenceArcPolicy.isIndependenceConsentError(lastError)
    }
}
