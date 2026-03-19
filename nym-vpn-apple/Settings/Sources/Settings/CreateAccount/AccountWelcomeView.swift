import Combine
import SwiftUI
import AppSettings
import Constants
import CredentialsManager
import ImpactGenerator
import ExternalLinkManager
import FeatureFlagsManager
#if os(iOS)
import PurchasesManager
#endif
import UIComponents
import Routes
import Theme

public struct AccountWelcomeView: View {
    private let navigationSource: AccountWelcomeNavigationSource

    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var credentialsManager: CredentialsManager
#if os(iOS)
    @EnvironmentObject private var purchasesManager: PurchasesManager
#endif
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager

    @Binding private var path: NavigationPath

    @State private var isDisplayingAlert = false
    @State private var alertTitle = ""
    @State private var isLoggingInWithPrivy = false
    private var type: AccountWelcomeType

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
            VStack(spacing: 0) {
                Spacer()
                createAccountTitle
                Spacer()
                createAccountSection
                Spacer()
                    .frame(height: 24)
                separatorLine
                Spacer()
                    .frame(height: 24)
                privySection
                Spacer()
                    .frame(height: 24)
                Spacer()
                TermsAndConditionsView()
                Spacer()
                    .frame(height: 24)
            }
            .frame(maxWidth: MagicNumbers.moreMaxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .alert(alertTitle, isPresented: $isDisplayingAlert) {
            Button("ok".localizedString, role: .cancel) { }
        }
        .onReceive(appSettings.$isCredentialImportedPublisher.removeDuplicates()) { newValue in
            guard newValue, isLoggingInWithPrivy else { return }
            isLoggingInWithPrivy = false
            navigateHome()
        }
    }

    public init(
        path: Binding<NavigationPath>,
        type: AccountWelcomeType,
        navigationSource: AccountWelcomeNavigationSource
    ) {
        _path = path
        self.type = type
        self.navigationSource = navigationSource
    }
}

private extension AccountWelcomeView {
    var navbar: some View {
        CustomNavBar(
            useElevationBackground: true,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    var createAccountTitle: some View {
        Text(accountLocalizedString)
            .textStyle(.Headline.Large.regular)
            .foregroundStyle(NymColor.primary)
    }

    var accountLocalizedString: String {
        switch type {
        case .createAccount:
            "createAccount".localizedString
        case .login:
            "login".localizedString
        }
    }

    var createAccountSection: some View {
        VStack(spacing: 8) {
            maximumPrivacyTitle
            maximumPrivacySubtitle
            Spacer()
                .frame(height: 8)
            createAccountButton
        }
        .padding(.horizontal, 24)
    }

    var maximumPrivacyTitle: some View {
        HStack {
            Text("🔒 \("createAccount.maximumPrivacy".localizedString)")
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var maximumPrivacySubtitle: some View {
        HStack {
            Text(privacySubtitleLocalizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Spacer()
        }
    }

    var privacySubtitleLocalizedString: String {
        switch type {
        case .createAccount:
            "\("createAccount.singleTap.subtitle".localizedString) \n\n\("createAccount.singleTap.subtitle1".localizedString)"
        case .login:
            "createAccount.Login.subtitle".localizedString
        }
    }

    var createAccountButton: some View {
#if os(iOS)
        GenericButton(title: createAccountButtonLocalizedTitle)
            .onTapGesture {
                navigateToLoginOrCreateAccount()
            }
            .accessibilityAction {
                navigateToLoginOrCreateAccount()
            }
#elseif os(macOS)
        GenericButton(title: createAccountButtonLocalizedTitle, isExternalLink: true)
            .onTapGesture {
                navigateToLoginOrCreateAccount()
            }
            .accessibilityAction {
                navigateToLoginOrCreateAccount()
            }
#endif
    }

    var createAccountButtonLocalizedTitle: String {
        switch type {
        case .createAccount:
            "createAccount.startAnonymously".localizedString
        case .login:
            "createAccount.loginAnonymously".localizedString
        }
    }

    var separatorLine: some View {
        Rectangle()
            .foregroundColor(NymColor.gray2)
            .frame(height: 1)
            .padding(.horizontal, 24)
    }

    var privySection: some View {
        VStack(spacing: 8) {
            privyTitle
            privySubtitle
            Spacer()
                .frame(height: 8)
            privyLoginButton
        }
        .padding(.horizontal, 24)
    }

    var privyTitle: some View {
        HStack {
            Text("⚡️ \("createAccount.quickSetup".localizedString)")
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var privySubtitle: some View {
        HStack {
            Text("createAccount.useExistingLogin.subtitle".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Spacer()
        }
    }

    var privyLoginButton: some View {
        GenericButton(
            title: privyLoginButtonTitle,
            style: .primaryBorderOnly
        )
        .onTapGesture {
            privyLogin()
        }
        .accessibilityAction {
            privyLogin()
        }
    }

    var privyLoginButtonTitle: String {
        switch type {
        case .login:
            "createAccount.loginWithSocialAccount".localizedString
        case .createAccount:
            "createAccount.continueOnSocial".localizedString
        }
    }
}

// MARK: - Actions -
private extension AccountWelcomeView {
    func navigateHome() {
        path = .init()
    }

    func navigateBack() {
        switch navigationSource {
        case .onboarding, .addCredential:
            path = .init([HomeLink.onboarding])
        case .settings:
            if !path.isEmpty { path.removeLast() }
        case .home:
            path = .init()
        }
    }

    func navigateToLoginOrCreateAccount() {
        switch type {
        case .createAccount:
            navigateToCreateAccount()
        case .login:
            navigateToLogin()
        }
    }

    func navigateToCreateAccount() {
#if os(iOS)
        ImpactGenerator.shared.impact()
        path.append(SettingLink.generatePassphrase(displayPurchaseView: false))
#elseif os(macOS)
        try? externalLinkManager.openExternalURL(urlString: Constants.pricingURL.rawValue)
        navigateToLogin()
#endif
    }

    func navigateToLogin() {
        ImpactGenerator.shared.impact()
        path.append(SettingLink.addCredentials(navigationSource: .accountWelcome))
    }

    func privyLogin() {
        isLoggingInWithPrivy = true
        ImpactGenerator.shared.impact()
        Task {
            do {
                let loginURL = try await credentialsManager.privyLogin()
                try externalLinkManager.openExternalURL(urlString: loginURL)
            } catch {
                alertTitle = error.localizedDescription
                isDisplayingAlert = true
            }
        }
    }
}

// MARK: - Helpers -
private extension AccountWelcomeView {
    func loginAttributedString() -> AttributedString? {
        let alreadyHaveAcccount = "createAccount.alreadyHaveAccount".localizedString
        let login = "createAccount.login".localizedString
        let loginLink = "login"
        return try? AttributedString(markdown: "\(alreadyHaveAcccount) [\(login)](\(loginLink))")
    }
}
