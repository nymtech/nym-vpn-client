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

                    buttons()
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
    func buttons() -> some View {
        HStack {
            Spacer()
            if let yesLocalizedString = viewModel.configuration.yesLocalizedString {
                yesButton(text: yesLocalizedString)
            }

            if let noLocalizedString = viewModel.configuration.noLocalizedString {
                Spacer()
                    .frame(width: 16)

                noButton(text: noLocalizedString)
            }

            Spacer()
        }
    }

    @ViewBuilder
    func yesButton(text: String) -> some View {
        GenericButton(title: text)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.success()
#endif
                viewModel.configuration.yesAction?()
                viewModel.isDisplayed = false
            }
    }

    @ViewBuilder
    func noButton(text: String) -> some View {
        GenericButton(title: text, borderOnly: true)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                viewModel.configuration.noAction?()
                viewModel.isDisplayed = false
            }
    }
}
