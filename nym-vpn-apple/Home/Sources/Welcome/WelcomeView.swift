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
        VStack {
            Spacer()
            titleView()
            subtitleView()
            sentryToggle()
                .frame(maxWidth: Device.type == .ipad ? 450 : .infinity)
            continueButton()
                .frame(maxWidth: Device.type == .ipad ? 450 : .infinity)
            privacyPolicy()
                .padding(.bottom, 24)
        }
        .frame(maxWidth: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension WelcomeView {
    @ViewBuilder
    func titleView() -> some View {
        Text(viewModel.titleText)
            .textStyle(.Headline.Small.primary)
            .multilineTextAlignment(.center)
        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitleView() -> some View {
        Text("\(Text(viewModel.subtitle1Text)) \(Text("[\(viewModel.sentryText)](https://sentry.io)"))\(Text(viewModel.subtitle2Text)) \n\n\(Text(viewModel.disclaimerText))")
            .textStyle(.Body.Large.regular)
            .tint(NymColor.primaryOrange)
            .foregroundStyle(NymColor.statusInfoText)
            .multilineTextAlignment(.center)
            .padding(.horizontal, viewModel.subtitleViewHorizontalPadding())
        Spacer()
    }

    @ViewBuilder
    func sentryToggle() -> some View {
        SettingsListItem(viewModel: viewModel.sentryViewModel())
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func continueButton() -> some View {
        GenericButton(title: viewModel.continueText)
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.continueTapped()
            }
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func privacyPolicy() -> some View {
        Text("By continuing, you agree to NymVPN's [Terms of use](https://nymvpn.com/en/terms) and acknowledge NymVPN's [Privacy policy](https://nymvpn.com/en/privacy?type=apps).")
            .tint(NymColor.sysOnSurfaceWhite)
            .foregroundStyle(NymColor.sysOutline)
            .textStyle(.Label.Medium.primary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }
}
