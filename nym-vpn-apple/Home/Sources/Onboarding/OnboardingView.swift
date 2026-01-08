import SwiftUI
import AppSettings
import UIComponents
import Theme

public struct OnboardingView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @State private var selection: Int = 0
    @State private var pageCount = 3

    public var body: some View {
        VStack(spacing: 0) {
            VStack(spacing: 0) {
                Spacer()
                    .frame(height: 36)
                logo()
                Spacer()
                onboardingStepsView()
                Spacer()
                createAccountButton()
                Spacer()
                    .frame(height: 24)
                loginButton()
#if os(macOS)
                Spacer()
                    .frame(height: 24)
#endif
            }
            .frame(maxWidth: MagicNumbers.maxWidth, alignment: .top)
            .padding(.horizontal, 24)
        }
        .frame(maxWidth: .infinity, alignment: .top)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init() {}

    var pages = [
        OnboardingStepView(
            imageName: "7daysNoCommitment",
            title: "onboarding0.title".localizedString,
            subtitle: "onboarding0.subtitle1".localizedString + "\n\n" + "onboarding0.subtitle2".localizedString
        ),
        OnboardingStepView(
            imageName: "speed",
            title: "onboarding3.title".localizedString,
            subtitle: "onboarding3.subtitle1".localizedString + "\n" + "onboarding3.subtitle2".localizedString
        ),
        OnboardingStepView(
            imageName: "stopBeingTracked",
            title: "onboarding1.title".localizedString,
            subtitle: "onboarding1.subtitle1".localizedString
        ),
        OnboardingStepView(
            imageName: "zeroKnowledge",
            title: "onboarding2.title".localizedString,
            subtitle: "onboarding2.subtitle1".localizedString
        )
    ]
}

private extension OnboardingView {
    func logo() -> some View {
        HStack(spacing: 0) {
            GenericImage(imageName: "logoText")
                .frame(width: 110, height: 30)
        }
    }

    @ViewBuilder
    func onboardingStepsView() -> some View {
        PagerView(pageCount: pages.count, currentIndex: $selection) { index in
            pages[index]
        }
        Spacer()
            .frame(height: 16)
        PageIndicator(pageCount: pages.count, selection: $selection)
    }

    func createAccountButton() -> some View {
        GenericButton(title: "createAccount".localizedString)
            .onTapGesture {
                navigateToCreateAccount()
            }
            .accessibilityAction {
                navigateToCreateAccount()
            }
    }

    func loginButton() -> some View {
        GenericButton(title: "login".localizedString, style: .primaryBorderOnly)
            .onTapGesture {
                navigateTologin()
            }
            .accessibilityAction {
                navigateTologin()
            }
    }
}

private extension OnboardingView {
    func navigateToCreateAccount() {
        appSettings.onboardingDidDisplay = true
    }

    func navigateTologin() {
        appSettings.onboardingDidDisplay = true
    }
}

#Preview {
    OnboardingView()
}
