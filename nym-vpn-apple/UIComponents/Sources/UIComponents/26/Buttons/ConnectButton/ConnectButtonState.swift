import SwiftUI
import AccountPrefetchGates
import Theme
import TunnelStatus

public enum ConnectButtonState: Equatable {
    case connect
    case disconnect
    case disconnecting
    case stop
    case installingDaemon
    case noInternet
    case noInternetReconnect
    case noAccount
    case noSubscription
    case accountUnreachable
    case checkingAccount

    public init(
        tunnelStatus: TunnelStatus,
        isCredentialImported: Bool,
        accountSummaryLastFetchFailed: Bool = false,
        isAccountActive: Bool = true,
        hasAccountSummary: Bool = false
    ) {
        if isCredentialImported == false {
            self = .noAccount
            return
        }
        switch tunnelStatus {
        case .connected:
            self = .disconnect
        case .connecting, .reasserting, .restarting:
            self = .stop
        case .disconnected:
            switch DisconnectedHomeCTA.resolve(
                isCredentialImported: isCredentialImported,
                accountSummaryLastFetchFailed: accountSummaryLastFetchFailed,
                isAccountActive: isAccountActive,
                hasAccountSummary: hasAccountSummary
            ) {
            case .getStarted:
                self = .noAccount
            case .choosePlan:
                self = .noSubscription
            case .accountUnreachable:
                self = .accountUnreachable
            case .checking:
                self = .checkingAccount
            case .connect:
                self = .connect
            }
        case .disconnecting:
            self = .disconnecting
        case .offline, .unknown:
            self = .noInternet
        case .offlineReconnect:
            self = .noInternetReconnect
        case .error:
            self = .stop
        }
    }

    public var localizedTitle: String {
        switch self {
        case .connect, .noInternet:
            "connect".localizedString
        case .disconnect:
            "disconnect".localizedString
        case .disconnecting:
            "disconnecting".localizedString
        case .stop, .noInternetReconnect:
            "stop".localizedString
        case .installingDaemon:
            "home.installDaemonButton".localizedString
        case .noAccount:
            "home.getStarted".localizedString
        case .noSubscription:
            "purchasePlan.chooseMyPlan".localizedString
        case .accountUnreachable:
            "home.accountUnreachable".localizedString
        case .checkingAccount:
            "requestingZkNyms".localizedString
        }
    }

    var backgroundColor: Color {
        switch self {
        case .connect, .noInternet, .noAccount, .noSubscription, .accountUnreachable:
            NymColor.accent
        case .installingDaemon, .noInternetReconnect, .checkingAccount:
            NymColor.gray1
        case .stop, .disconnecting, .disconnect:
            NymColor.error
        }
    }
}

#if os(macOS)
extension ConnectButtonState {
    public var menuBarItemIsAction: Bool {
        switch self {
        case .connect, .disconnect, .stop, .noInternetReconnect, .noInternet:
            true
        case .disconnecting, .installingDaemon, .noAccount, .noSubscription, .accountUnreachable, .checkingAccount:
            false
        }
    }
}
#endif
