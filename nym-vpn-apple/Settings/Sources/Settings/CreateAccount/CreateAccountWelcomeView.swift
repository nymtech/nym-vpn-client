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

public struct CreateAccountWelcomeView: View {
    private let navigationSource: CreateAccountNavigationSource

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
                if featureFlagsManager.isPrivyEnabled {
                    privySection
                    Spacer()
                        .frame(height: 24)
                }
                alreadyHaveAnAccount
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

    public init(path: Binding<NavigationPath>, navigationSource: CreateAccountNavigationSource) {
        _path = path
        self.navigationSource = navigationSource
    }
}

private extension CreateAccountWelcomeView {
    var navbar: some View {
        CustomNavBar(
            useElevationBackground: true,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    var createAccountTitle: some View {
        Text("createAccount".localizedString)
            .textStyle(.Headline.Large.regular)
            .foregroundStyle(NymColor.primary)
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
            Text("⚡️ \("createAccount.instantAndAnonymous".localizedString)")
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var maximumPrivacySubtitle: some View {
        HStack {
            Text("createAccount.singleTap.subtitle".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Spacer()
        }
    }

    var createAccountButton: some View {
#if os(iOS)
        GenericButton(title: "createAccount.startAnonymously".localizedString)
            .onTapGesture {
                navigateToCreateAccount()
            }
            .accessibilityAction {
                navigateToCreateAccount()
            }
#elseif os(macOS)
        GenericButton(title: "createAccount.startAnonymously".localizedString, isExternalLink: true)
            .onTapGesture {
                navigateToCreateAccount()
            }
            .accessibilityAction {
                navigateToCreateAccount()
            }
#endif
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
            Text("🔑 \("createAccount.useExistingLogin".localizedString)")
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
            title: "createAccount.continueOnSocial".localizedString,
            style: .primaryBorderOnly,
            isLoading: $isLoggingInWithPrivy
        )
        .onTapGesture {
            privyLogin()
        }
        .accessibilityAction {
            privyLogin()
        }
    }

    @ViewBuilder var alreadyHaveAnAccount: some View {
        if let loginAttributedString = loginAttributedString() {
            Text(loginAttributedString)
                .tint(NymColor.accent)
                .textStyle(.Body.Large.regular)
                .multilineTextAlignment(.center)
                .foregroundStyle(NymColor.gray1)
                .padding(.bottom, 24)
                .environment(\.openURL, OpenURLAction { url in
                    guard url.absoluteString == "login" else { return .discarded }
                    navigateToLogin()
                    return .handled
                })
        }
    }
}

// MARK: - Actions -
private extension CreateAccountWelcomeView {
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
        path.append(SettingLink.addCredentials(navigationSource: .createAccountWelcome))
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
private extension CreateAccountWelcomeView {
    func loginAttributedString() -> AttributedString? {
        let alreadyHaveAcccount = "createAccount.alreadyHaveAccount".localizedString
        let login = "createAccount.login".localizedString
        let loginLink = "login"
        return try? AttributedString(markdown: "\(alreadyHaveAcccount) [\(login)](\(loginLink))")
    }
}
