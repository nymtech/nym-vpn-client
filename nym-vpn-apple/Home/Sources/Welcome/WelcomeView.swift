import SwiftUI
import Constants
import Device
import Theme
import UIComponents

public struct WelcomeView: View {
    @ObservedObject var viewModel: WelcomeViewModel

    public init(viewModel: WelcomeViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        ZStack {
            NymColor.background
                .ignoresSafeArea()

            VStack(spacing: 0) {
                Spacer()
                titleView()
                subtitleView()
                sentryToggle()
#if os(macOS)
                statisticsToggle()
#endif
                Spacer()
                    .frame(height: 24)
                continueButton()
                privacyPolicy()
                    .padding(.bottom, 24)
            }
            .padding(.horizontal, 16)
            .frame(minWidth: 375, maxWidth: Device.type == .ipad ? 450 : 500)
        }
    }
}

private extension WelcomeView {
    @ViewBuilder
    func titleView() -> some View {
        Text(viewModel.titleText)
            .textStyle(.Headline.Large.regular)
            .multilineTextAlignment(.center)
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitleView() -> some View {
        Text(viewModel.subtitleAttributedString() ?? "")
            .textStyle(.Body.Large.regular)
            .tint(NymColor.accent)
            .foregroundStyle(NymColor.gray1)
            .multilineTextAlignment(.center)
            .padding(.horizontal, viewModel.subtitleViewHorizontalPadding())
        Spacer()
    }

    func sentryToggle() -> some View {
        SettingsListItem(viewModel: viewModel.sentryViewModel())
    }

    func statisticsToggle() -> some View {
        SettingsListItem(viewModel: viewModel.statisticsViewModel())
    }

    @ViewBuilder
    func continueButton() -> some View {
        GenericButton(title: viewModel.continueText)
            .onTapGesture {
                viewModel.continueTapped()
            }
            .accessibilityAction {
                viewModel.continueTapped()
            }
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func privacyPolicy() -> some View {
        Text(viewModel.privacyPolicyAttributedString() ?? "")
            .tint(NymColor.primary)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Small.regular)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }
}
