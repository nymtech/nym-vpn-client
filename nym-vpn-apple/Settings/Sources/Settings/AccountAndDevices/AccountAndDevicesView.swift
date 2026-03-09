import SwiftUI
import AppSettings
import ConnectionTypes
import Constants
import ImpactGenerator
import ConfigurationManager
import ConnectionManager
import CredentialsManager
import ExternalLinkManager
import UIComponents
import Theme

@MainActor public struct AccountAndDevicesView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var configurationManager: ConfigurationManager
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager

    @State private var isPresentedManageSubscription = false
    @State private var isLogoutConfirmationDisplayed = false
    @State private var isLogoutLoading = false

    @Binding private var path: NavigationPath

    @State private var isLinkAccountAvailable = false

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
private extension AccountAndDevicesView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.account".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    @ViewBuilder
    func renewButton() -> some View {
        if let accountSummary = credentialsManager.accountSummary, !accountSummary.isActive {
            GenericButton(title: "settings.account.renewNow".localizedString)
                .onTapGesture {
                    navigateToPlanPurchase()
                }
        }
    }

    @ViewBuilder
    func accountStatusSection() -> some View {
        if let accountSummary = credentialsManager.accountSummary {
            VStack(spacing: 0) {
                accountStatusHeader()
                if let accountSummary = credentialsManager.accountSummary {
                    accountStatusBandwidth(accountSummary: accountSummary)
                    Divider()
                        .frame(height: 1)
                        .overlay(NymColor.background)
                        .padding(.horizontal, 16)
                    accountStatusResetDate(accountSummary: accountSummary)
                    renewNowRow(accountSummary: accountSummary)
                } else {
                    Divider()
                        .frame(height: 1)
                        .overlay(NymColor.background)
                    accountStatusInactive()
                }
            }
            .background(NymColor.elevation)
            .clipShape(RoundedRectangle(cornerRadius: 8))
        }
    }

    func accountStatusInactive() -> some View {
        VStack(spacing: 16) {
            ZStack {
                Circle()
                    .stroke(NymColor.gray1, lineWidth: 2)
                    .frame(width: 64, height: 64)
                GenericImage(systemImageName: "shield.slash")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(NymColor.gray1)
            }
            Text("settings.account.noActivePlan".localizedString)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 32)
    }

    func accountStatusHeader() -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 16)
            HStack(spacing: 8) {
                GenericImage(systemImageName: "gauge.with.dots.needle.50percent")
                    .frame(width: 20, height: 20)
                    .foregroundStyle(NymColor.gray1)
                Text("settings.account.status".localizedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Large.regular)
                Spacer()
            }
            .padding(.horizontal, 16)
            Spacer()
                .frame(height: 16)
        }
    }

    func accountStatusBandwidth(accountSummary: AccountSummary) -> some View {
        VStack(spacing: 8) {
            HStack {
                Text("settings.account.bandwidthRemaining".localizedString)
                    .foregroundStyle(NymColor.accent)
                    .textStyle(.Body.Small.regular)
                Spacer()
                Text("settings.account.bandwidthLimit".localizedString)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
            }

            bandwidthProgressBar(
                used: accountSummary.trafficUsedGb,
                limit: accountSummary.trafficLimitGb,
                color: NymColor.accent
            )

            HStack {
                Text(bandwidthRemainingText(used: accountSummary.trafficUsedGb, limit: accountSummary.trafficLimitGb))
                    .foregroundStyle(NymColor.accent)
                    .textStyle(.Body.Small.regular)
                Spacer()
                Text(bandwidthLimitText(limit: accountSummary.trafficLimitGb))
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    func accountStatusResetDate(accountSummary: AccountSummary) -> some View {
        HStack {
            Text("settings.account.resetsOn".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
            Text(resetDateText(date: accountSummary.trafficResetDate))
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Medium.regular)
        }
        .padding(.horizontal, 16)
        .frame(height: 48)
    }

    @ViewBuilder
    func renewNowRow(accountSummary: AccountSummary) -> some View {
        if !accountSummary.isAutoRenewEnabled {
            let color = accountSummary.statusColor
            Button {
                navigateToAccount()
            } label: {
                HStack(spacing: 8) {
                    GenericImage(imageName: "bolt")
                        .frame(width: 16, height: 16)
                        .foregroundStyle(color)
                    Text("settings.account.renewNow".localizedString)
                        .foregroundStyle(color)
                        .textStyle(.Body.Medium.regular)
                    Spacer()
                    GenericImage(imageName: "externalLink")
                        .frame(width: 16, height: 16)
                        .foregroundStyle(color)
                }
                .padding(.horizontal, 16)
                .frame(height: 48)
                .background(color.opacity(0.15))
            }
            .buttonStyle(.plain)
        }
    }

    func bandwidthProgressBar(used: Int?, limit: Int?, color: Color) -> some View {
        GeometryReader { geometry in
            let remaining = max(0, (limit ?? 0) - (used ?? 0))
            let fraction = limit.map { $0 > 0 ? CGFloat(remaining) / CGFloat($0) : 0 } ?? 0

            ZStack(alignment: .leading) {
                RoundedRectangle(cornerRadius: 4)
                    .fill(NymColor.gray2)
                    .frame(height: 8)

                RoundedRectangle(cornerRadius: 4)
                    .fill(color)
                    .frame(width: geometry.size.width * fraction, height: 8)
            }
        }
        .frame(height: 8)
    }

    func bandwidthRemainingText(used: Int?, limit: Int?) -> String {
        let remaining = max(0, (limit ?? 0) - (used ?? 0))
        return formatBandwidth(remaining)
    }

    func bandwidthLimitText(limit: Int?) -> String {
        formatBandwidth(limit ?? 0)
    }

    func formatBandwidth(_ gb: Int) -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .decimal
        formatter.maximumFractionDigits = 0
        let formatted = formatter.string(from: NSNumber(value: gb)) ?? "\(gb)"
        return "\(formatted) GB"
    }

    func resetDateText(date: Date?) -> String {
        guard let date else { return "-" }
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy.MM.dd"
        return formatter.string(from: date)
    }

    func contactSupportText() -> some View {
        HStack(spacing: 0) {
            Text(contactSupportAttributedString())
                .tint(NymColor.gray1)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
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

    func contactSupportAttributedString() -> AttributedString {
        let bolt = AttributedString("⚡ ")
        var link = AttributedString("settings.account.contactSupport.link".localizedString)
        link.underlineStyle = .single
        link.foregroundColor = NymColor.primary
        link.link = URL(string: Constants.supportURL.rawValue)
        let suffix = AttributedString(" \("settings.account.contactSupport.suffix".localizedString)")
        return bolt + link + suffix
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
}

// MARK: - Views -
private extension AccountAndDevicesView {
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

    var logoutDialogConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "rectangle.portrait.and.arrow.right",
            titleLocalizedString: "settings.logoutTitle".localizedString,
            subtitleLocalizedString: "settings.logoutSubtitle".localizedString,
            yesLocalizedString: "settings.logout".localizedString,
            noLocalizedString: "cancel".localizedString,
            isYesDestructive: true,
            yesAction: {
                isLogoutLoading = true
                Task {
                    await logout()
                    try? await Task.sleep(for: .seconds(0.3))
                    Task { @MainActor in
                        isLogoutConfirmationDisplayed = false
                        isLogoutLoading = false
                        navigateBack()
                    }
                }
            },
            loadingText: "settings.loggingOut".localizedString,
            shouldCloseAfterYesAction: false,
            verticalButtonsLayout: true
        )
    }
}

// MARK: - Helpers -
private extension AccountAndDevicesView {
    func updateIsAccountLinkAvailable() async {
        await credentialsManager.updateAccountSummary()
        guard let accountSummary = credentialsManager.accountSummary else { return }
        isLinkAccountAvailable = !accountSummary.isLinked
    }
}

// MARK: - Actions -
private extension AccountAndDevicesView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func navigateToAccount() {
        impactGenerator.softImpact()
        try? externalLinkManager.openExternalURL(urlString: configurationManager.accountLinks?.account)
    }

    func navigateToPlanPurchase() {
        impactGenerator.softImpact()
#if os(iOS)
        path.append(SettingLink.generatePassphrase(displayPurchaseView: true))
#elseif os(macOS)
        try? externalLinkManager.openExternalURL(urlString: configurationManager.accountLinks?.account)
#endif
    }

    func linkAccount() async {
        impactGenerator.softImpact()
        let link = try? await credentialsManager.privyLogin(isLink: true)
        try? externalLinkManager.openExternalURL(urlString: link)
    }

    func logout() async {
        await connectionManager.disconnectBeforeLogout()
        try? await credentialsManager.removeCredential()
    }
}
