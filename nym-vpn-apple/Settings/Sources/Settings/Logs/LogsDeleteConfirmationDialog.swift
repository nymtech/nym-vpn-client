import SwiftUI
import Theme
import UIComponents

struct LogsDeleteConfirmationDialog: View {
    @ObservedObject private var viewModel: LogsDeleteConfirmationDialogViewModel

    init(viewModel: LogsDeleteConfirmationDialogViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
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

private extension LogsDeleteConfirmationDialog {
    @ViewBuilder
    func icon() -> some View {
        Spacer()
            .frame(height: 24)

        Image(systemName: viewModel.trashIconImageName)
            .frame(width: 24, height: 24)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text(viewModel.deleteAllLogsLocalizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.sysOnSurface)
            .multilineTextAlignment(.center)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitle() -> some View {
        Text(viewModel.cannotRetrieveLogsLocalizedString)
            .foregroundStyle(NymColor.modeInfoViewDescription)
            .textStyle(.Body.Medium.regular)
            .multilineTextAlignment(.center)
            .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 24))
    }

    @ViewBuilder
    func yesButton() -> some View {
        GenericButton(title: viewModel.yesLocalizedString)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.success()
#endif
                viewModel.action()
            }
    }

    @ViewBuilder
    func noButton() -> some View {
        GenericButton(title: viewModel.noLocalizedString, borderOnly: true)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                viewModel.isDisplayed = false
            }
    }
}
