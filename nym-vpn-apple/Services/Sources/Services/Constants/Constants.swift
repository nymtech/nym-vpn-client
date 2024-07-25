import Foundation

public enum Constants: String {
    case groupID = "group.net.nymtech.vpn"
    case helperName = "net.nymtech.vpn.helper"

    case supportURL = "https://support.nymvpn.com/hc/en-us"
    case termsOfUseURL = "https://nymvpn.com/en/terms"
    case privacyPolicyURL = "https://nymvpn.com/en/privacy?type=apps"
    case emailLink = "https://support.nymvpn.com/hc/en-us/requests/new"

    case discordLink = "https://nymtech.net/go/discord"
    case ghIssuesLink = "https://www.nymtech.net/go/github/nym-vpn-client/issues"

    case logFileName = "Logs.log"

    public static let currentEnvironment: Env = .canary

    public static func apiURL() -> URL? {
        getenv("NYM_API").flatMap { URL(string: String(cString: $0)) }
    }
}

public enum Env: String {
    case canary
    case mainnet
    case sandbox
}
