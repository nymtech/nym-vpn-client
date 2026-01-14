import SwiftUI
import Constants
import ImpactGenerator
import ExternalLinkManager
#if os(iOS)
import PurchasesManager
#endif
import UIComponents
import Routes
import Theme

public struct CreateAccountWelcomeView: View {
    private let navigationSource: CreateAccountNavigationSource
#if os(iOS)
    @EnvironmentObject private var purchasesManager: PurchasesManager
#endif
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @Binding private var path: NavigationPath

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
            VStack(spacing: 0) {
                Spacer()
                createAccountTitle
                Spacer()
                    .frame(height: 24)
                createAccountSection
                Spacer()
                    .frame(height: 24)
                separatorLine
                Spacer()
                    .frame(height: 24)
                alreadyHaveAnAccount
                Spacer()
                    .frame(height: 24)
                TermsAndConditionsView()
                Spacer()
            }
            .frame(maxWidth: MagicNumbers.moreMaxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
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
            Text("🔒 \("createAccount.maximumPrivacy".localizedString)")
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var maximumPrivacySubtitle: some View {
        HStack {
            Text("createAccount.maximumPrivacy.subtitle".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Spacer()
        }
    }

    var createAccountButton: some View {
#if os(iOS)
        GenericButton(title: "createAccount.createAccountButtonTitle".localizedString)
            .onTapGesture {
                navigateToCreateAccount()
            }
            .accessibilityAction {
                navigateToCreateAccount()
            }
#elseif os(macOS)
        GenericButton(title: "createAccount.createAccountButtonTitle".localizedString, isExternalLink: true)
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
