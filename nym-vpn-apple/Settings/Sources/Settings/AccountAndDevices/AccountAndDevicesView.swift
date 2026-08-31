import SwiftUI
import AppSettings
import AccountPrefetchGates
import ConnectionTypes
import Constants
import ImpactGenerator
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
#if os(iOS)
import PurchasesManager
#endif
import SnackbarManager
import TunnelStatus
import UIComponents
import Theme

@MainActor public struct AccountAndDevicesView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject var configurationManager: ConfigurationManager
    @EnvironmentObject var connectionManager: ConnectionManager
    @EnvironmentObject var credentialsManager: CredentialsManager
    @EnvironmentObject var impactGenerator: ImpactGenerator
    @EnvironmentObject var externalLinkManager: ExternalLinkManager
#if os(iOS)
    @EnvironmentObject var purchasesManager: PurchasesManager
#endif

    @State private var isPresentedManageSubscription = false
    @State var isLogoutConfirmationDisplayed = false
    @State var isLogoutLoading = false
    @State var logoutProgressText: String? = nil
    @State var isRefreshingAccount = false
    @State var autologinState = AutologinState()

    @Binding private var path: NavigationPath

    public var body: some View {
        GeometryReader { geometry in
            VStack(spacing: 0) {
                navbar()
                Spacer()
                    .frame(height: 24)
                ScrollView {
                    VStack(spacing: 24) {
                        if credentialsManager.isValidCredentialImported {
                            accountStatusSection()
                            nymAccountSection()
                            accountIdentifier()
                            accountIdText()
                            deviceIdentifier()
                            deviceIdText()
#if os(iOS)
                            if !configurationManager.isTestFlight {
                                manageSubscription()
                            }
#endif
                            if appSettings.isCredentialImported {
                                logoutButton()
                            }
                        }
                    }
                    .frame(maxWidth: MagicNumbers.maxWidth)
                    .padding(.horizontal, 16)
                    Spacer()
                        .frame(height: max(geometry.safeAreaInsets.bottom, 24))
                }
                .scrollIndicators(.never)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .ignoresSafeArea(edges: [.bottom])
        }
        .navigationBarBackButtonHidden(true)
#if os(iOS)
        .manageSubscriptionsSheet(isPresented: $isPresentedManageSubscription)
#endif
        .autologinOverlay(state: autologinState, onRetry: { navigateToAccount() })
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .overlay {
            if isLogoutConfirmationDisplayed {
                ActionDialogView(
                    viewModel: ActionDialogViewModel(
                        isDisplayed: $isLogoutConfirmationDisplayed,
                        configuration: logoutDialogConfiguration,
                        impactGenerator: .shared,
                        isLoading: $isLogoutLoading,
                        loadingTextOverride: $logoutProgressText
                    )
                )
            }
        }
        .task {
            await credentialsManager.updateAccountSummary()
            showAllowanceReachedSnackbarIfNeeded()
        }
        .onChange(of: credentialsManager.didReceiveSubscriptionPayment) { _, received in
            guard received else { return }
            autologinState.dismissAfterWebReturn()
            Task {
                await credentialsManager.updateAccountSummary()
                showAllowanceReachedSnackbarIfNeeded()
            }
        }
        .onChange(of: credentialsManager.didReceiveAccountLinkCallback) { _, _ in
            Task {
                await credentialsManager.updateAccountSummary()
                showAllowanceReachedSnackbarIfNeeded()
            }
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Views -
extension AccountAndDevicesView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.account".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func nymAccountSection() -> some View {
        manageAccountListItem(isFirst: true, isLast: true)
    }

    func manageAccountListItem(isFirst: Bool, isLast: Bool) -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .externalLink,
                title: "settings.account.manageSubscription".localizedString,
                attributtedSubtitle: manageSubscriptionSubtitle(),
                imageName: "eventRepeat",
                position: SettingsListItemPosition(isFirst: isFirst, isLast: isLast),
                action: {
                    navigateToAccount()
                }
            )
        )
    }

    func manageSubscriptionSubtitle() -> AttributedString? {
        let accountSummary = credentialsManager.accountSummary
        if accountSummary?.subscription?.status == .pending {
            var confirmingPayment = AttributedString("confirmingPayment".localizedString)
            confirmingPayment.foregroundColor = Color.Nym.error
            return confirmingPayment
        } else {
            return credentialsManager.accountSummary?.planValidUntilAttributedString
        }
    }

    @ViewBuilder
    func accountIdentifier() -> some View {
        if let accountIdentifier = credentialsManager.accountIdentifier {
            cell(
                title: "settings.accountID".localizedString,
                subtitle: accountIdentifier,
                systemImageName: "number",
                imageSize: 16
            )
        }
    }

    func accountIdText() -> some View {
        HStack(spacing: 0) {
            Text(accountIdAttributedString())
                .tint(Color.Nym.textSecondary)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
        .environment(\.openURL, OpenURLAction { url in
            if url.absoluteString == Constants.supportURL.rawValue {
                try? externalLinkManager.openExternalURL(urlString: url.absoluteString)
                return .handled
            }
            return .systemAction
        })
    }

    func accountIdAttributedString() -> AttributedString {
        let prefix = AttributedString("settings.account.accountId".localizedString)
        var link = AttributedString("settings.account.accountId.supportLink".localizedString)
        link.underlineStyle = .single
        link.foregroundColor = Color.Nym.textSecondary
        link.link = URL(string: Constants.supportURL.rawValue)
        return prefix + link
    }

    @ViewBuilder
    func deviceIdentifier() -> some View {
        if let deviceIdentifier = credentialsManager.deviceIdentifier {
            cell(
                title: "settings.deviceId".localizedString,
                subtitle: deviceIdentifier,
                systemImageName: "macbook.and.iphone",
                imageSize: 24
            )
        }
    }

    func deviceIdText() -> some View {
        HStack(spacing: 0) {
            Text("settings.account.deviceId".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            Spacer()
        }
    }

    func manageSubscription() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .externalLink,
                title: "settings.manageSubscription".localizedString,
                systemImageName: "creditcard",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    impactGenerator.softImpact()
                    isPresentedManageSubscription = true
                }
            )
        )
    }

    func logoutButton() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "settings.logout".localizedString,
                type: .destructive,
                position: .init(isFirst: true, isLast: true),
                action: {
                    isLogoutConfirmationDisplayed = true
                }
            )
        )
    }

    func cell(title: String, subtitle: String, systemImageName: String, imageSize: CGFloat) -> some View {
        SettingsCopyableContentCell(
            title: title,
            subtitle: subtitle,
            systemImageName: systemImageName,
            imageSize: imageSize,
            onCopy: {
#if os(iOS)
                UIPasteboard.general.string = subtitle
                ImpactGenerator.shared.impact()
#elseif os(macOS)
                NSPasteboard.general.prepareForNewContents()
                NSPasteboard.general.setString(subtitle, forType: .string)
#endif
            }
        )
    }
}

// MARK: - Actions -
extension AccountAndDevicesView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func navigateToRoot() {
        path = .init()
    }

    func navigateToAccount() {
        impactGenerator.softImpact()
        autologinState.start(kind: .autologinView, using: credentialsManager)
    }

    func navigateToPlanPurchase() {
        impactGenerator.softImpact()
#if os(iOS)
        path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
#elseif os(macOS)
        autologinState.start(kind: .autologinRenew, using: credentialsManager)
#endif
    }

    /// Surfaces the daily-allowance-reached error snackbar (critical = red, white, no
    /// close button) once the summary reports the quota is spent.
    func showAllowanceReachedSnackbarIfNeeded() {
        guard credentialsManager.accountSummary?.isDailyAllowanceReached == true else { return }
        SnackbarManager.shared.enqueue(
            SnackbarItem(
                style: .critical,
                title: "settings.account.allowanceReached.title".localizedString,
                message: "settings.account.allowanceReached.subtitle".localizedString
            )
        )
    }

    func refreshAccount() {
        guard !isRefreshingAccount else { return }
        impactGenerator.softImpact()
        isRefreshingAccount = true
        Task {
            defer { isRefreshingAccount = false }
            do {
                try await credentialsManager.refreshAccountSummary()
            } catch {
                SnackbarManager.shared.enqueue(
                    SnackbarItem(
                        style: .negative,
                        title: "settings.account.refreshFailed".localizedString
                    )
                )
            }
        }
    }

    @MainActor
    func logout() async {
        await credentialsManager.beginLogout()
        defer { credentialsManager.endLogout() }

        if LogoutTeardownPolicy.needsDisconnectWait(for: connectionManager.currentTunnelStatus) {
            logoutProgressText = "disconnecting".localizedString
        }
        await connectionManager.disconnectBeforeLogout()

        logoutProgressText = "settings.loggingOut".localizedString
        try? await credentialsManager.removeCredential()
        logoutProgressText = nil
    }
}
