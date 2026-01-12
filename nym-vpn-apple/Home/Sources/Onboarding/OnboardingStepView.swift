import SwiftUI
import UIComponents
import Theme

public struct OnboardingStepView: View {
    let imageName: String
    let title: String
    let subtitle: String

    public var body: some View {
        VStack(spacing: 0) {
            onboardingImage()
                .frame(minHeight: 120, maxHeight: 300)
            Spacer()
                .frame(height: 16)
            onboardingTitle()
            Spacer()
                .frame(height: 16)
            onboardingSubtitle()
        }
    }
}

private extension OnboardingStepView {
    func onboardingImage() -> some View {
        GenericImage(imageName: imageName)
    }

    func onboardingTitle() -> some View {
        Text(title)
            .foregroundStyle(.primary)
            .textStyle(.Headline.Large.regular)
            .multilineTextAlignment(.center)
    }

    func onboardingSubtitle() -> some View {
        Text(subtitle)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Medium.regular)
            .multilineTextAlignment(.center)
    }
}

#Preview {
    OnboardingStepView(
        imageName: "dataMixing",
        title: "Title",
        subtitle: "Subtitle"
    )
}
