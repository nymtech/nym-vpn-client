#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
#endif

public enum AccountStatus: Equatable, Hashable {
    case active
    case inactive
    case deleteMe

    init(vpnAccountStatus: VpnAccountStatus) {
        switch vpnAccountStatus {
        case .active:
            self = .active
        case .inactive:
            self = .inactive
        case .deleteMe:
            self = .deleteMe
        }
    }
}
