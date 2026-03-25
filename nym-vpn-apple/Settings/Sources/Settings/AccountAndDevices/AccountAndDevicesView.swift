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
    @State var isPinCodeDisplayed = false
    @State var isLinkAccountAvailable = false
    @State var pinCode: String = ""
    @State var isAutologinLoading = false
    @State var isAutologinError = false
    @State var autologinErrorMessage = ""
    @State var autologinURL = ""
    @State var autologinTask: Task<Void, Never>?

    @Binding private var path: NavigationPath

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            ScrollView {
                VStack(spacing: 24) {
                    if credentialsManager.isValidCredentialImported {
                        renewButton()
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
            }
            .scrollIndicators(.never)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
#if os(iOS)
        .manageSubscriptionsSheet(isPresented: $isPresentedManageSubscription)
#endif
        .overlay {
            if isPinCodeDisplayed, !pinCode.isEmpty {
                PinCodeView(
                    isDisplayed: $isPinCodeDisplayed,
                    pinCode: $pinCode,
                    url: $autologinURL
                )
            }
        }
        .background {
            NymColor.background
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
        .overlay {
            if isAutologinLoading {
                ActionDialogView(
                    viewModel: ActionDialogViewModel(
                        isDisplayed: $isAutologinLoading,
                        configuration: autologinLoadingConfiguration,
                        impactGenerator: .shared,
                        isLoading: .constant(true)
                    )
                )
            }
        }
        .overlay {
            if isAutologinError {
                ActionDialogView(
                    viewModel: ActionDialogViewModel(
                        isDisplayed: $isAutologinError,
                        configuration: autologinErrorConfiguration,
                        impactGenerator: .shared
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
    func renewButton() -> some View {
        if let accountSummary = credentialsManager.accountSummary,
           accountSummary.shouldShowRenewButton(isAutoRenew: isAutoRenewEnabled(accountSummary: accountSummary)) {
            GenericButton(title: accountSummary.renewButtonTitle)
                .onTapGesture {
                    navigateToPlanPurchase()
                }
        }
    }

    private func isAutoRenewEnabled(accountSummary: AccountSummary) -> Bool {
#if os(iOS)
        purchasesManager.isAutoRenewEnabled || accountSummary.isAutoRenewEnabled
#elseif os(macOS)
        accountSummary.isAutoRenewEnabled
#endif
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
        credentialsManager.accountSummary?.planValidUntilAttributedString
    }

    func nymLinkingText() -> some View {
        HStack(spacing: 0) {
            Text(linkingTitle())
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
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
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
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
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
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
        isAutologinLoading = true

        autologinTask = Task {
            await autologin(kind: .autologinView)
        }
    }

    func navigateToPlanPurchase() {
        impactGenerator.softImpact()
#if os(iOS)
        path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
#elseif os(macOS)
        isAutologinLoading = true

        autologinTask = Task {
            await autologin(kind: .autologinRenew)
        }
#endif
    }

    func autologin(kind: NymDeeplinkKind) async {
        do {
            guard let result = try await credentialsManager.autologin(kind: kind) else {
                isAutologinLoading = false
                return
            }
            isAutologinLoading = false
            pinCode = result.pinCode
            autologinURL = result.url
            isPinCodeDisplayed = true
        } catch is CancellationError {
            isAutologinLoading = false
        } catch {
            isAutologinLoading = false
            autologinErrorMessage = error.localizedDescription
            isAutologinError = true
        }
    }

    func linkAccount() async {
        impactGenerator.softImpact()
        let link = try? await credentialsManager.privyLogin(kind: .privyLink)
        try? externalLinkManager.openExternalURL(urlString: link)
    }

    func logout() async {
        await connectionManager.disconnectBeforeLogout()
        try? await credentialsManager.removeCredential()
    }
}
