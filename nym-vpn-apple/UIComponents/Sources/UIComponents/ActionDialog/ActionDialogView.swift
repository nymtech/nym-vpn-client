import SwiftUI
import Theme

public struct ActionDialogView: View {
    @ObservedObject private var viewModel: ActionDialogViewModel

    public init(viewModel: ActionDialogViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            HStack {
                Spacer()
                    .frame(width: 40)

                VStack {
                    icon()
                    Spacer()
                        .frame(height: 16)
                    title()
                    subtitle()
                    HStack {
                        Spacer()
                        yesButton()

                        Spacer()
                            .frame(width: 16)

                        noButton()
                        Spacer()
                    }
                    .padding(24)
                }
                .background(NymColor.modeInfoViewBackground)
                .cornerRadius(16)

                Spacer()
                    .frame(width: 40)
            }
        }
        .edgesIgnoringSafeArea(.all)
    }
}

private extension ActionDialogView {
    @ViewBuilder
    func icon() -> some View {
        if let iconImageName = viewModel.configuration.iconImageName {
            Spacer()
                .frame(height: 24)

            Image(systemName: iconImageName)
                .frame(width: 24, height: 24)
        }
    }

    @ViewBuilder
    func title() -> some View {
        if let title = viewModel.configuration.titleLocalizedString {
            Text(title)
                .textStyle(NymTextStyle.Label.Huge.bold)
                .foregroundStyle(NymColor.sysOnSurface)
                .multilineTextAlignment(.center)

            Spacer()
                .frame(height: 16)
        }
    }

    @ViewBuilder
    func subtitle() -> some View {
        if let subtitle = viewModel.configuration.subtitleLocalizedString {
            Text(subtitle)
                .foregroundStyle(NymColor.modeInfoViewDescription)
                .textStyle(.Body.Medium.regular)
                .multilineTextAlignment(.center)
                .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 24))
        }
    }

    @ViewBuilder
    func yesButton() -> some View {
        GenericButton(title: viewModel.configuration.yesLocalizedString)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.success()
#endif
                viewModel.configuration.yesAction?()
                viewModel.isDisplayed = false
            }
    }

    @ViewBuilder
    func noButton() -> some View {
        GenericButton(title: viewModel.configuration.noLocalizedString, borderOnly: true)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                viewModel.configuration.noAction?()
                viewModel.isDisplayed = false
            }
    }
}
