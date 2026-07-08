import SwiftUI
import Theme
import UIComponents

struct OnboardingStepView: View {
    let step: OnboardingStep

    @State private var speedMode: OneClickSpeedMode = .fast

    var body: some View {
        VStack(spacing: 0) {
            illustration
                .frame(maxWidth: .infinity)
                .frame(height: Constants.illustrationHeight)
            Spacer()
                .frame(height: NymSpacing.section)
            title
            Spacer()
                .frame(height: NymSpacing.large)
            subtitle
        }
        .frame(maxWidth: .infinity)
    }
}

private extension OnboardingStepView {
    @ViewBuilder
    var illustration: some View {
        switch step {
        case .modes:
            SpeedModeSegmentedControl(selection: speedMode) { mode in
                speedMode = mode
            }
            .frame(maxWidth: Constants.segmentedControlWidth)
        default:
            if let imageName = step.imageName {
                GenericImage(imageName: imageName)
                    .accessibilityHidden(true)
            }
        }
    }

    var title: some View {
        Text(step.titleKey.localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var subtitle: some View {
        Text(step.subtitle)
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textSecondary)
            .multilineTextAlignment(.center)
    }

    enum Constants {
        static let illustrationHeight: CGFloat = 300
        static let segmentedControlWidth: CGFloat = 350
    }
}

#if DEBUG
#Preview {
    OnboardingStepView(step: .censorship)
        .background(Color.Nym.background)
}
#endif
