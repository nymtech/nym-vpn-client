import Foundation
import NymVPNLib

public struct AccountAuthMethod: Codable {
    public var id: String
    public var pubkey: String
    public var kind: String
    public var label: String
    public var status: AccountStatus
    public var created: Date

    init(id: String, pubkey: String, kind: String, label: String, status: AccountStatus, created: Date) {
        self.id = id
        self.pubkey = pubkey
        self.kind = kind
        self.label = label
        self.status = status
        self.created = created
    }

    public init(vpnAccountMethod: VpnAccountAuthMethod) {
        self.id = vpnAccountMethod.id
        self.pubkey = vpnAccountMethod.pubkey
        self.kind = vpnAccountMethod.kind
        self.label = vpnAccountMethod.label
        self.status = AccountStatus(vpnAccountStatus: vpnAccountMethod.status)
        self.created = Date(timeIntervalSince1970: TimeInterval(vpnAccountMethod.created))
    }
}
