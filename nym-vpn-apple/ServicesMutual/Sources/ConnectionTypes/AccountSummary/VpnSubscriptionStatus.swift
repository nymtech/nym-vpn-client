import NymVPNLib

public enum VpnSubscriptionStatus: Equatable, Hashable, Codable {
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
