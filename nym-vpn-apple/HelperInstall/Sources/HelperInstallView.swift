import SwiftUI
import Theme
import MessageModels
import UIComponents

public struct HelperInstallView: View {
    @ObservedObject var viewModel: HelperInstallViewModel

    public init(viewModel: HelperInstallViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
            explanationText()
            firstStepText()
            openSystemSettingsButton()
            secondStepText()
            secondStepImage()
            thirdStepText()
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .overlay {
            succesfullyInstalledModal()
        }
        .overlay {
            migrationModal()
        }
        .onAppear {
            viewModel.registerDaemonIfNeeded()
        }
        // Copy to clipboard success message
        .snackbar(
            isDisplayed: $viewModel.isSnackBarDisplayed,
            message: SnackBarMessage(text: "viewModel.copiedSuccesfullyMessage", style: .info)
        )
    }
}

extension HelperInstallView {
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.navTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    func explanationText() -> some View {
        HStack {
            Text(viewModel.infoText)
                .textStyle(.Body.Large.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.leading)
            Spacer()
        }
        .padding(EdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 16))
    }

    func firstStepText() -> some View {
        HStack {
            Text("helper.installView.firstStep".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.leading)
            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16))
    }

    func openSystemSettingsButton() -> some View {
        GenericButton(
            title: "helper.installView.openSystemSettings".localizedString,
            height: 40,
            isWidthExpanded: false
        )
        .padding(.bottom, 24)
        .onTapGesture {
            viewModel.openSystemSettings()
        }
    }

    func secondStepText() -> some View {
        HStack {
            Text("helper.installView.secondStep".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.leading)
            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16))
    }

    func secondStepImage() -> some View {
        GenericImage(imageName: "daemonSystemSettings")
            .frame(maxWidth: 450)
            .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16))
    }

    @ViewBuilder
    func thirdStepText() -> some View {
        if let thirdStepText = viewModel.thirdStepAttributedString() {
            HStack {
                Text("3. \(thirdStepText)")
                    .tint(NymColor.action)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)
                    .multilineTextAlignment(.leading)
                Spacer()
            }
            .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16))
        }
    }

    @ViewBuilder
    func succesfullyInstalledModal() -> some View {
        if viewModel.isSuccessModalDisplayed {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $viewModel.isSuccessModalDisplayed,
                    configuration: viewModel.updateAvailableOverlayConfiguration,
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isSuccessModalDisplayed)
        }
    }

    @ViewBuilder
    func migrationModal() -> some View {
        if viewModel.isMigrationModalDisplayed {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $viewModel.isMigrationModalDisplayed,
                    configuration: viewModel.migrationOverlayConfiguration,
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isMigrationModalDisplayed)
        }
    }
}
