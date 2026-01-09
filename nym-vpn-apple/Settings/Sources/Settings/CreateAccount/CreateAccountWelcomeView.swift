import SwiftUI
import ImpactGenerator
import PurchasesManager
import UIComponents
import Routes
import Theme

public struct CreateAccountWelcomeView: View {
    private let navigationSource: CreateAccountNavigationSource
#if os(iOS)
    @EnvironmentObject private var purchasesManager: PurchasesManager
#endif
    @Binding private var path: NavigationPath

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
            VStack(spacing: 0) {
                logoView
                Spacer()
                    .frame(height: 40)
                createAccountTitle
                Spacer()
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
            isLogoImageHidden: true,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    var logoView: some View {
        GenericImage(imageName: "logoText")
            .frame(height: 24)
            .accessibilityLabel("NymVPN".localizedString)
            .accessibilityAddTraits([.isImage])
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
        GenericButton(title: "createAccount.createAccountButtonTitle".localizedString)
            .onTapGesture {
                navigateToCreateAccount()
            }
            .accessibilityAction {
                navigateToCreateAccount()
            }
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
        case .onboarding:
            path = .init([HomeLink.onboarding])
        case .settings:
            if !path.isEmpty { path.removeLast() }
        case .home:
            path = .init()
        }
    }

    func navigateToCreateAccount() {
        ImpactGenerator.shared.impact()
        path.append(SettingLink.generatePassphrase)
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
