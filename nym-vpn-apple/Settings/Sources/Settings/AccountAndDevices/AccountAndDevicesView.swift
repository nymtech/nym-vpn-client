import SwiftUI
import AppSettings
import ConnectionTypes
import Constants
import ImpactGenerator
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
import PurchasesManager
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
    @State var isLinkAccountAvailable = false
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
                            contactSupportText()
                            nymAccountSection()
                            nymLinkingText()
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
                        isLoading: $isLogoutLoading
                    )
                )
            }
        }
        .task {
            await updateIsAccountLinkAvailable()
        }
        .onChange(of: credentialsManager.didReceiveAccountLinkCallback) { _, _ in
            Task {
                await updateIsAccountLinkAvailable()
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

    @ViewBuilder
    func nymAccountSection() -> some View {
        VStack(spacing: 0) {
            if isLinkAccountAvailable {
                manageAccountListItem(isFirst: true, isLast: false)
                SettingsListItem(
                    viewModel: SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.account.nymAccount".localizedString,
                        subtitle: accountSubtitle(),
                        imageName: "person",
                        position: SettingsListItemPosition(isFirst: false, isLast: true),
                        action: {
                            Task {
                                await linkAccount()
                            }
                        }
                    )
                )
            } else {
                manageAccountListItem(isFirst: true, isLast: true)
            }
        }
    }

    func accountSubtitle() -> String? {
        guard let accountSummary = credentialsManager.accountSummary else { return nil }
        return accountSummary.isLinked ? nil : "settings.account.nymAccount.subtitle".localizedString
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

    func nymLinkingText() -> some View {
        HStack(spacing: 0) {
            Text(linkingTitle())
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            Spacer()
        }
    }

    func linkingTitle() -> String {
        guard let accountSummary = credentialsManager.accountSummary else { return "" }
        return accountSummary.isLinked
        ? "⚡️ \("settings.account.nymAccount.linked.subtitle".localizedString)"
        : "⚠️ \("settings.account.linking".localizedString)"
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
            Text("settings.account.accountId".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            Spacer()
        }
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

// MARK: - Helpers -
extension AccountAndDevicesView {
    func updateIsAccountLinkAvailable() async {
        await credentialsManager.updateAccountSummary()
        guard let accountSummary = credentialsManager.accountSummary else { return }
        isLinkAccountAvailable = !accountSummary.isLinked
    }
}

// MARK: - Actions -
extension AccountAndDevicesView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
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

    func linkAccount() async {
        impactGenerator.softImpact()
        let link = try? await credentialsManager.privyLogin(kind: .privyLink)
#if os(iOS)
        try? externalLinkManager.openInAppBrowser(urlString: link)
#else
        try? externalLinkManager.openExternalURL(urlString: link)
#endif
    }

    func logout() async {
        await connectionManager.disconnectBeforeLogout()
        try? await credentialsManager.removeCredential()
    }
}
