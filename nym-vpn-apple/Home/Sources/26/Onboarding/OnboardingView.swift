import SwiftUI
import ImpactGenerator
import Theme
import UIComponents

public struct OnboardingView: View {
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @State private var selection = 0

    private let onGetStarted: () -> Void

    public init(onGetStarted: @escaping () -> Void) {
        self.onGetStarted = onGetStarted
    }

    public var body: some View {
        VStack(spacing: 0) {
            Spacer(minLength: NymSpacing.large)
            pager
            Spacer(minLength: NymSpacing.large)
            PageIndicator(pageCount: OnboardingStep.allCases.count, selection: $selection)
                .padding(.horizontal, NymSpacing.large)
            Spacer()
                .frame(height: NymSpacing.section)
            bottomCard
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .overlay(alignment: .topTrailing) {
            closeButton
        }
    }
}

private extension OnboardingView {
    var closeButton: some View {
        ImageButton(
            systemImageName: "xmark",
            imageSize: Constants.closeIconSize,
            accessibilityLabel: "close".localizedString
        ) {
            impactGenerator.softImpact()
            onGetStarted()
        }
        .padding(.top, NymSpacing.small)
        .padding(.trailing, NymSpacing.large)
    }

    var pager: some View {
        PagerView(pageCount: OnboardingStep.allCases.count, currentIndex: $selection) { index in
            OnboardingStepView(step: OnboardingStep.allCases[index])
        }
        .frame(maxWidth: NymSpacing.drawerMaxWidth)
    }

    /// Steps without a tagline keep a blank line so the card does not resize between pages.
    @ViewBuilder
    var tagline: some View {
        let taglineKey = OnboardingStep.allCases[selection].taglineKey
        Group {
            if let taglineKey {
                Text(taglineKey.localizedString)
            } else {
                Text(verbatim: " ")
                    .accessibilityHidden(true)
            }
        }
        .nymTextStyle(.bodyDefault)
        .foregroundStyle(Color.Nym.textSecondary)
        .multilineTextAlignment(.center)
        .lineLimit(1, reservesSpace: true)
        .minimumScaleFactor(Constants.taglineMinimumScale)
    }

    var bottomCard: some View {
        VStack(spacing: NymSpacing.medium) {
            GenericImage(imageName: "logoText")
                .frame(width: Constants.logoWidth, height: Constants.logoHeight)
                .accessibilityHidden(true)
            tagline
            NymButton("onboarding26.getStarted".localizedString, style: .primary) {
                impactGenerator.softImpact()
                onGetStarted()
            }
        }
        .padding(NymSpacing.component)
        .frame(maxWidth: NymSpacing.drawerMaxWidth)
        .background {
            RoundedRectangle(cornerRadius: Constants.cardCornerRadius)
                .fill(Color.Nym.surface)
        }
        .padding(.horizontal, NymSpacing.large)
        .padding(.bottom, NymSpacing.section)
    }

    enum Constants {
        static let logoWidth: CGFloat = 100
        static let logoHeight: CGFloat = 27
        static let cardCornerRadius: CGFloat = 16
        static let closeIconSize: CGFloat = 24
        static let taglineMinimumScale: CGFloat = 0.8
    }
}

#if DEBUG
#Preview {
    OnboardingView(onGetStarted: {})
}
#endif
