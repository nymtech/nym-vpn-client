import SwiftUI
#if os(iOS)
import ImpactGenerator
#endif
import UIComponents
import Theme

public struct CreateAccountWelcomeView: View {
    @Binding private var path: NavigationPath

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
            VStack(spacing: 0) {
                backgroundDots
                Spacer()
                    .frame(height: 40)
                welcomeTitle
                nymVpnTitle
                Spacer()
                    .frame(height: 24)
                benefitsList
                Spacer()
                    .frame(height: 32)
                createAccountButton
                Spacer()
                    .frame(height: 32)
                alreadyHaveAnAccount
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

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension CreateAccountWelcomeView {
    var navbar: some View {
        CustomNavBar(
            useElevationBackground: true,
            isLogoImageHidden: true,
            leftButton: CustomNavBarButton(type: .back, action: { navigateHome() })
        )
    }

    var logoView: some View {
        GenericImage(imageName: "logoText")
            .frame(height: 24)
            .accessibilityLabel("NymVPN".localizedString)
            .accessibilityAddTraits([.isImage])
    }

    var backgroundDots: some View {
        ZStack {
            GenericImage(imageName: "createAccountWelcomeDots")
            logoView
        }
    }

    var welcomeTitle: some View {
        Text("addCredentials.welcome.Title".localizedString)
            .textStyle(.Headline.ExtraLarge.bold)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }

    var nymVpnTitle: some View {
        Text("NymVPN".localizedString)
            .textStyle(.Headline.ExtraLarge.bold)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }

    var benefitsList: some View {
        VStack(alignment: .leading, spacing: 8) {
            benefitListItem(titleKey: "createAccount.anonymousMixnetTechnology", imageName: "verifiedUser")
            benefitListItem(titleKey: "createAccount.countries", imageName: "world")
            benefitListItem(titleKey: "createAccount.unlikedData", imageName: "accountBalance")
            benefitListItem(titleKey: "createAccount.openSource", imageName: "code")
        }
    }

    func benefitListItem(titleKey: String, imageName: String) -> some View {
        HStack(spacing: 0) {
            GenericImage(imageName: imageName)
                .frame(width: 20, height: 20)
                .foregroundStyle(NymColor.accent)
            Spacer()
                .frame(width: 8)
            Text(titleKey.localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
        }
    }

    var createAccountButton: some View {
        GenericButton(title: "createAccount.createAccountButtonTitle".localizedString)
            .padding(.horizontal, 16)
            .onTapGesture {
                navigateToCreateAccount()
            }
            .accessibilityAction {
                navigateToCreateAccount()
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

    func navigateToCreateAccount() {
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        path.append(SettingLink.createAccount)
    }

    func navigateToLogin() {
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        path.append(SettingLink.addCredentials)
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
