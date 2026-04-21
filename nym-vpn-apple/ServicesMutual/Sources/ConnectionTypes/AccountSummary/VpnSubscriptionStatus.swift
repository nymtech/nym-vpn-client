#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public enum VpnSubscriptionStatus: Equatable, Hashable {
    case pending
    case active

    public init(from status: NymVpnSubscriptionStatus) {
        switch status {
        case .pending:
            self = .pending
        case .active:
            self = .active
        }
    }
}
