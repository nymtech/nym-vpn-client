import SwiftUI
import Constants
import ImpactGenerator
import Theme
import UIComponents

public struct WelcomeView: View {
    private let minHeight: CGFloat
    private let onSignInTapped: () -> Void
    private let onSignUpTapped: () -> Void

    public init(
        minHeight: CGFloat = 0,
        onSignInTapped: @escaping () -> Void,
        onSignUpTapped: @escaping () -> Void
    ) {
        self.minHeight = minHeight
        self.onSignInTapped = onSignInTapped
        self.onSignUpTapped = onSignUpTapped
    }

    public var body: some View {
        VStack(spacing: 0) {
            logo
            Spacer(minLength: NymSpacing.large)
            VStack(spacing: AuthLayout.stackSpacing) {
                heading
                subtitle
                buttons
            }
            Spacer(minLength: NymSpacing.large)
            tosFooter
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
        .frame(maxWidth: .infinity)
        .frame(minHeight: minHeight)
    }
}

private extension WelcomeView {
    var logo: some View {
        GenericImage(imageName: "logoText")
            .frame(width: 100, height: 27)
    }

    var heading: some View {
        Text("welcome.heading".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var subtitle: some View {
        Text("welcome.subtitle".localizedString)
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textSecondary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, NymSpacing.component)
    }

    var buttons: some View {
        VStack(spacing: NymSpacing.component) {
            NymButton("welcome.signUp".localizedString, style: .primary) {
                ImpactGenerator.shared.softImpact()
                onSignUpTapped()
            }
            NymButton("welcome.signIn".localizedString, style: .primary) {
                ImpactGenerator.shared.softImpact()
                onSignInTapped()
            }
        }
    }

    var tosFooter: some View {
        Text(tosAttributedString)
            .nymTextStyle(.bodySmall)
            .foregroundStyle(Color.Nym.textSecondary)
            .tint(Color.Nym.primary)
            .multilineTextAlignment(.center)
    }

    var tosAttributedString: AttributedString {
        let prefix = AttributedString("welcome.tos.prefix".localizedString)
        var terms = AttributedString("welcome.tos.terms".localizedString)
        terms.font = .Nym.bodySmallBold
        terms.link = URL(string: Constants.termsOfUseURL.rawValue)
        let and = AttributedString("welcome.tos.and".localizedString)
        var privacyPolicy = AttributedString("welcome.tos.privacyPolicy".localizedString)
        privacyPolicy.font = .Nym.bodySmallBold
        privacyPolicy.link = URL(string: Constants.privacyPolicyURL.rawValue)
        return prefix + terms + and + privacyPolicy
    }
}

#if DEBUG
#Preview {
    WelcomeView(onSignInTapped: {}, onSignUpTapped: {})
        .background(Color.Nym.surface)
}
#endif
