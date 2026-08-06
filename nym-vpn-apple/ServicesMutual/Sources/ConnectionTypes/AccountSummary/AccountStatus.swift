import NymVPNLib

public enum AccountStatus: Equatable, Hashable, Codable {
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
