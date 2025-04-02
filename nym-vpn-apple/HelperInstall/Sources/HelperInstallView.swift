import SwiftUI
import Theme
import UIComponents

public struct HelperInstallView: View {
    @State private var animationScaleSize: CGFloat = 1

    @ObservedObject var viewModel: HelperInstallViewModel

    public init(viewModel: HelperInstallViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
            Spacer()
            explanationText()
            Spacer()
            allStepsView()
            Spacer()
            errorMessage()
            Spacer()
            actionButton()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        // Copy to clipboard success message
        .snackbar(
            isDisplayed: $viewModel.isSnackBarDisplayed,
            style: .info,
            message: viewModel.copiedSuccesfullyMessage
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
        Text(viewModel.infoText)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.modeInfoViewTitle)
            .multilineTextAlignment(.center)
            .padding(16)
    }

    @ViewBuilder
    func actionButton() -> some View {
        GenericButton(title: viewModel.buttonTitle(), mainColor: viewModel.buttonColor())
            .padding(EdgeInsets(top: 0, leading: 16, bottom: 24, trailing: 16))
            .onTapGesture {
                viewModel.buttonAction()
            }
    }

    @ViewBuilder
    func allStepsView() -> some View {
        VStack(alignment: .leading) {
            ForEach(viewModel.steps, id: \.self) { step in
                stepView(step: step)
                Spacer()
                    .frame(height: 16)
            }
        }
    }

    @ViewBuilder
    func stepView(step: HelperInstallStep) -> some View {
        switch step {
        case .uninstallOldDeamon:
            Text(step.title)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.modeInfoViewTitle)
                .padding(.horizontal, 16)
            Spacer()
                .frame(height: 16)

            GenericButton(title: "helper.installView.copy".localizedString, borderOnly: true)
                .padding(.horizontal, 16)
                .onTapGesture {
                    viewModel.copyCommands()
                }
        default:
            HStack {
                PulsingImageView(systemImageName: step.systemImageName, imageColor: step.imageColor)
                Text(step.title)
                    .textStyle(.Body.Large.regular)
                    .lineLimit(3)
                    .foregroundStyle(NymColor.modeInfoViewTitle)
            }
            .padding(.horizontal, 16)
        }
    }

    @ViewBuilder
    func errorMessage() -> some View {
        if let message = viewModel.error?.localizedDescription {
            Text(message)
                .textStyle(.Body.Large.regular)
                .foregroundStyle(NymColor.sysError)
        }
    }
}
