import SwiftUI
import ImpactGenerator
import ConfigurationManager
import CredentialsManager
import ExternalLinkManager
import UIComponents
import Theme

@MainActor public struct AccountAndDevicesView: View {
    @EnvironmentObject private var configurationManager: ConfigurationManager
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager

    @State private var isPresentedManageSubscription = false

    @Binding private var path: NavigationPath

    @State private var isLinkAccountAvailable = false

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            VStack(spacing: 24) {
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
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            .padding(.horizontal, 16)
            Spacer()
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
        .task {
            await updateIsAccountLinkAvailable()
        }
        .onChange(of: credentialsManager.didReceiveAccountLinkCallback) { _, newValue in
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
    func nymAccountSection() -> some View {
        VStack(spacing: 0) {
            if isLinkAccountAvailable {
                SettingsListItem(
                    viewModel: SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.account.manageAccount".localizedString,
                        systemImageName: "cloud",
                        position: SettingsListItemPosition(isFirst: true, isLast: false),
                        action: {
                            navigateToAccount()
                        }
                    )
                )
                SettingsListItem(
                    viewModel: SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.account.nymAccount".localizedString,
                        subtitle: "settings.account.nymAccount.subtitle".localizedString,
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
                SettingsListItem(
                    viewModel: SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.account.manageAccount".localizedString,
                        systemImageName: "cloud",
                        position: SettingsListItemPosition(isFirst: true, isLast: true),
                        action: {
                            navigateToAccount()
                        }
                    )
                )
            }
        }
    }

    func nymLinkingText() -> some View {
        HStack(spacing: 0) {
            Text("⚠️ \("settings.account.linking".localizedString)")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Spacer()
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
}

// MARK: - Helpers -
private extension AccountAndDevicesView {
    func updateIsAccountLinkAvailable() async {
        if let isAvailable = try? await credentialsManager.isAccountLinkAvailable() {
            isLinkAccountAvailable = isAvailable
        }
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

    func linkAccount() async {
        impactGenerator.softImpact()
        let link = try? await credentialsManager.privyLogin(isLink: true)
        try? externalLinkManager.openExternalURL(urlString: link)
    }
}
