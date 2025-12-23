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
                nextButton()
                skipButton()
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
            imageName: "dataMixing",
            title: "onboarding1.title".localizedString,
            subtitle: "onboarding1.subtitle1".localizedString + "\n\n" + "onboarding1.subtitle2".localizedString
        ),
        OnboardingStepView(
            imageName: "zeroKnowledge",
            title: "onboarding2.title".localizedString,
            subtitle: "onboarding2.subtitle1".localizedString + "\n\n" + "onboarding2.subtitle2".localizedString
        ),
        OnboardingStepView(
            imageName: "speed",
            title: "onboarding3.title".localizedString,
            subtitle: "onboarding3.subtitle1".localizedString + "\n\n" + "onboarding3.subtitle2".localizedString
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
        PageIndicator(pageCount: 3, selection: $selection)
    }

    func nextButton() -> some View {
        GenericButton(title: "onboarding.next".localizedString)
            .onTapGesture {
                nextAction()
            }
            .accessibilityAction {
                nextAction()
            }
    }

    func skipButton() -> some View {
        GenericButton(title: "onboarding.skip".localizedString, style: .borderless)
            .onTapGesture {
                skipAction()
            }
            .accessibilityAction {
                skipAction()
            }
    }
}

private extension OnboardingView {
    func nextAction() {
        if selection < pageCount - 1 {
            selection += 1
        } else {
            appSettings.onboardingDidDisplay = true
        }
    }

    func skipAction() {
        appSettings.onboardingDidDisplay = true
    }
}

#Preview {
    OnboardingView()
}
