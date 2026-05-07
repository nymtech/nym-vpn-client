import SwiftUI
import Theme
import UIComponents

struct LogsDeleteConfirmationDialog: View {
    @ObservedObject private var viewModel: LogsDeleteConfirmationDialogViewModel

    init(viewModel: LogsDeleteConfirmationDialogViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        ModalOverlayView(isDisplayed: $viewModel.isDisplayed, dismissOnOverlayTap: false) {
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
        }
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
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitle() -> some View {
        Text(viewModel.cannotRetrieveLogsLocalizedString)
            .foregroundStyle(Color.Nym.textSecondary)
            .nymTextStyle(.bodyDefault)
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
        GenericButton(title: viewModel.noLocalizedString, style: .accentBorderOnly)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.impact()
#endif
                viewModel.isDisplayed = false
            }
    }
}
