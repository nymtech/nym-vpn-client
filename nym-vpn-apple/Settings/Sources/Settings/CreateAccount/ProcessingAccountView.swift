import SwiftUI
import AppSettings
import Routes
import Theme
import UIComponents

public struct ProcessingAccountView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding private var path: NavigationPath
    @State private var didFinishAnimatingText = false
    @State private var currentStep = 1

    public var body: some View {
        VStack(alignment: .center, spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)

            StepView(stepCount: 4, currentStep: $currentStep)
            Spacer()
            dotsAnimationView
            Spacer()
                .frame(height: 16)
            animatingTextView
            Spacer()
        }
        .frame(maxWidth: MagicNumbers.moreMaxWidth)
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .onChange(of: didFinishAnimatingText) { _, _ in
            navigateHomeOrTechnicalOptIns()
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension ProcessingAccountView {
    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var dotsAnimationView: some View {
        WaveDotsView()
    }

    var animatingTextView: some View {
        SwitchingTitlesView(
            pairs: [
                ("processingAccount.title2".localizedString, "processingAccount.subtitle2".localizedString),
                ("processingAccount.title3".localizedString, "processingAccount.subtitle3".localizedString),
                ("processingAccount.title4".localizedString, "processingAccount.subtitle4".localizedString),
                ("processingAccount.title5".localizedString, "processingAccount.subtitle5".localizedString)
            ],
            didFinishAnimating: $didFinishAnimatingText,
            timerDidTick: {
                currentStep += 1
            }
        )
    }
}

private extension ProcessingAccountView {
    func navigateHomeOrTechnicalOptIns() {
        if appSettings.welcomeScreenDidDisplay {
            path = .init()
        } else {
            path = .init([HomeLink.technicalOptIns])
        }

    }
}
