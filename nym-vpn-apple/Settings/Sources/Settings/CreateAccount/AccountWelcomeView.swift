import AuthenticationServices
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
    @State private var isCreateAccount = false
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
            Color.Nym.background
                .ignoresSafeArea()
        }
        .alert(alertTitle, isPresented: $isDisplayingAlert) {
            Button("ok".localizedString, role: .cancel) { }
        }
        .onReceive(appSettings.$isCredentialImportedPublisher.removeDuplicates()) { newValue in
            guard newValue else { return }
            if isLoggingInWithPrivy {
                isLoggingInWithPrivy = false
                navigateHome()
            } else if isCreateAccount {
                isCreateAccount = false
                navigateHome()
            }
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
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
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
                .nymTextStyle(.bodyLarge)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }
    }

    var maximumPrivacySubtitle: some View {
        HStack {
            Text(privacySubtitleLocalizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
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
        GenericButton(title: createAccountButtonLocalizedTitle)
            .onTapGesture {
                isCreateAccount = true
                navigateToLoginOrCreateAccount()
            }
            .accessibilityAction {
                isCreateAccount = true
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
            .foregroundColor(Color.Nym.textSecondary)
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
                .nymTextStyle(.bodyLarge)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }
    }

    var privySubtitle: some View {
        HStack {
            Text("createAccount.useExistingLogin.subtitle".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            Spacer()
        }
    }

    var privyLoginButton: some View {
        GenericButton(
            title: privyLoginButtonTitle,
            style: .primaryBorderOnly,
            isExternalLink: true
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
            path = .init()
        case .settings:
            if !path.isEmpty { path.removeLast() }
        case .home:
            path = .init()
        }
    }

    func navigateToLoginOrCreateAccount() {
        switch type {
        case .createAccount:
            Task {
                await navigateToCreateAccount()
            }
        case .login:
            navigateToLogin()
        }
    }

    func navigateToCreateAccount() async {
#if os(iOS)
        ImpactGenerator.shared.impact()
        path.append(SettingLink.generatePassphrase(displayPurchaseView: false))
#elseif os(macOS)
        let createAccountLink = try? await credentialsManager.privyLogin(kind: .createAccount)
        try? await externalLinkManager.presentPrivyAuthSession(urlString: createAccountLink)
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
                let loginURL = try await credentialsManager.privyLogin(kind: .privy)
                try await externalLinkManager.presentPrivyAuthSession(urlString: loginURL)
            } catch let error as ASWebAuthenticationSessionError where error.code == .canceledLogin {
                return
            } catch {
                alertTitle = error.localizedDescription
                isDisplayingAlert = true
            }
        }
    }
}
