import SwiftUI
#if os(iOS)
import PurchasesManager
#endif
import Theme
import UIComponents

struct OnboardingStepView: View {
    #if os(iOS)
    @EnvironmentObject private var purchasesManager: PurchasesManager
    #endif

    let step: OnboardingStep

    var body: some View {
        VStack(spacing: 0) {
            if step.speedMode != nil {
                title
                Spacer()
                    .frame(height: NymSpacing.section)
            }
            // Flexible height: the illustration absorbs the squeeze on short screens, the copy never truncates.
            illustration
                .frame(maxWidth: .infinity, maxHeight: step.illustrationHeight)
            Spacer()
                .frame(height: NymSpacing.section)
            if step.speedMode == nil {
                title
                Spacer()
                    .frame(height: NymSpacing.large)
            }
            subtitle
            if let speedMode = step.speedMode {
                Spacer()
                    .frame(height: NymSpacing.section)
                SpeedModeSegmentedControl(selection: speedMode) { _ in }
                    .frame(maxWidth: Constants.segmentedControlWidth)
                    .padding(.horizontal, NymSpacing.section)
                    .allowsHitTesting(false)
            }
        }
        .frame(maxWidth: .infinity)
    }
}

private extension OnboardingStepView {
    @ViewBuilder
    var illustration: some View {
        switch step {
        case .plan:
            planIllustration
        case .welcome, .dvpn, .mixnet, .censorship:
            if let animationName = step.animationName {
                LoopAnimationView(animationName: animationName, fillColor: Color.Nym.textPrimary)
                    .accessibilityHidden(true)
            } else if let imageName = step.imageName {
                GenericImage(imageName: imageName)
                    .accessibilityHidden(true)
            }
        }
    }

    var planIllustration: some View {
        GenericImage(imageName: "logoText")
            .frame(width: Constants.planLogoWidth, height: Constants.planLogoHeight)
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background {
                planGlow
            }
            .accessibilityHidden(true)
    }

    var planGlow: some View {
        ZStack {
            glowCircle
                .offset(x: -Constants.glowOffset, y: Constants.glowOffset / 2)
            glowCircle
                .offset(x: Constants.glowOffset, y: -Constants.glowOffset)
            glowCircle
                .offset(x: Constants.glowOffset * 0.8, y: Constants.glowOffset)
        }
        .blur(radius: Constants.glowBlurRadius)
        .allowsHitTesting(false)
    }

    var glowCircle: some View {
        Circle()
            .fill(Color.Nym.primary.opacity(Constants.glowOpacity))
            .frame(width: Constants.glowDiameter, height: Constants.glowDiameter)
    }

    var title: some View {
        Text(step.titleKey.localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
            .textCase(step.isTitleUppercased ? .uppercase : nil)
            .fixedSize(horizontal: false, vertical: true)
            .padding(.horizontal, NymSpacing.section)
    }

    var pricing: OnboardingPlanPricing? {
        #if os(iOS)
        OnboardingPlanPricing(purchasesManager: purchasesManager)
        #else
        OnboardingPlanPricing()
        #endif
    }

    var subtitle: some View {
        Text(step.subtitle(pricing: pricing))
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textSecondary)
            .multilineTextAlignment(.center)
            .fixedSize(horizontal: false, vertical: true)
            .padding(.horizontal, NymSpacing.section)
    }

    enum Constants {
        static let segmentedControlWidth: CGFloat = 350
        static let planLogoWidth: CGFloat = 192
        static let planLogoHeight: CGFloat = 52
        static let glowDiameter: CGFloat = 160
        static let glowOffset: CGFloat = 130
        static let glowBlurRadius: CGFloat = 60
        static let glowOpacity: CGFloat = 0.35
    }
}

#if DEBUG
#Preview {
    {
        let view = OnboardingStepView(step: .plan)
            .background(Color.Nym.background)

        #if os(iOS)
        return view.environmentObject(PurchasesManager())
        #else
        return view
        #endif
    }()
}
#endif

