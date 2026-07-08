import Foundation

/// Pure decisions for iOS logout VPN teardown (bounded wait after manual disconnect).
public enum LogoutTeardownPolicy: Equatable, Sendable {
    /// Rust `DISCONNECT_TIMEOUT` (5s) plus Network Extension status slack.
    public static let disconnectWaitCapSeconds: TimeInterval = 7

    public static func needsDisconnectWait(for status: TunnelStatus) -> Bool {
        status != .disconnected
    }

    /// When the tunnel is already tearing down, do not call `stopVPNTunnel` again.
    public static func shouldInitiateDisconnect(for status: TunnelStatus) -> Bool {
        switch status {
        case .connected, .connecting, .reasserting, .restarting, .offlineReconnect, .offline, .error:
            return true
        case .disconnecting, .disconnected, .unknown:
            return false
        }
    }

    /// Profile reset assumes the tunnel reached `.disconnected` within the logout wait cap.
    public static func shouldResetVpnProfileAfterLogoutDisconnect(disconnectedInTime: Bool) -> Bool {
        disconnectedInTime
    }
}
