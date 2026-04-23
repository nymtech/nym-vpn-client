import NetworkExtension

public enum VPNStatus {
    case status(NEVPNStatus)
    case error
    case notConfigured

    public var isConnecting: Bool {
        switch self {
        case .status(let status):
            return status == .connecting
        default:
            return false
        }
    }

    public var isDisconnecting: Bool {
        switch self {
        case .status(let status):
            return status == .disconnecting
        default:
            return false
        }
    }

    public var isConnected: Bool {
        switch self {
        case .status(let status):
            return status == .connected || status == .reasserting
        default:
            return false
        }
    }

    /// Init from TunnelStatus.rawValue stored in UserDefaults (macOS).
    /// TunnelStatus cases: connected=0, connecting=1, disconnected=2, disconnecting=3, error=4,
    public init(tunnelStatusRawValue: Int) {
        switch tunnelStatusRawValue {
        case 0:
            self = .status(.connected)
        case 1:
            self = .status(.connecting)
        case 2:
            self = .status(.disconnected)
        case 3:
            self = .status(.disconnecting)
        case 4:
            self = .error
        default: self = .status(.disconnected)
        }
    }
}
