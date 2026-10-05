import SwiftUI
import Theme
import UIComponents

public struct ProfilesView: View {
    @StateObject private var viewModel: ProfilesViewModel

    public init(viewModel: ProfilesViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer().frame(height: NymSpacing.section)
            scrollViewContent()
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.background.ignoresSafeArea()
        }
    }
}

private extension ProfilesView {
    func navbar() -> some View {
        CustomNavBar(
            title: "profiles.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: viewModel.navigateBack)
        )
    }

    func scrollViewContent() -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: NymSpacing.section) {
                Text("profiles.subtitle".localizedString)
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textSecondary)

                VStack(spacing: NymSpacing.extraExtraSmall) {
                    ForEach(viewModel.profiles, id: \.self) { profile in
                        ProfileRow(
                            imageName: profile.imageName,
                            title: profile.titleKey.localizedString,
                            description: profile.descriptionKey.localizedString,
                            isSelected: profile == viewModel.currentProfile
                        ) {
                            viewModel.select(profile)
                        }
                    }
                }

                learnMoreLink()
            }
        }
        .scrollIndicators(.never)
        .frame(maxWidth: NymSpacing.contentWidth)
        .padding(.horizontal, NymSpacing.large)
    }

    @ViewBuilder
    func learnMoreLink() -> some View {
        HStack(spacing: 4) {
            Text("profiles.learnMore".localizedString)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(Color.Nym.textPrimary)
                .underline()

            GenericImage(imageName: "externalLink")
                .frame(width: 12, height: 12)
                .padding(4)
                .foregroundStyle(Color.Nym.textPrimary)

            Spacer()
        }
        .onTapGesture {
            viewModel.learnMore()
        }
        .accessibilityAction {
            viewModel.learnMore()
        }
    }
}
