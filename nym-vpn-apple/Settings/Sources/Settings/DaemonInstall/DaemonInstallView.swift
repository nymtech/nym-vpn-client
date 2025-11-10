#if os(macOS)
import SwiftUI
import Constants
import Theme
import MessageModels
import ServiceManagement
import UIComponents

public struct DaemonInstallView: View {
    @Binding private var path: NavigationPath
    @Binding private var isServing: Bool {
        didSet {
            guard isServing else { return }
            isSuccessModalDisplayed = true
        }
    }
    @State private var isSuccessModalDisplayed = false

    public init(
        isServing: Binding<Bool>,
        path: Binding<NavigationPath>
    ) {
        _isServing = isServing
        _path = path
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
    }
}

extension DaemonInstallView {
    func navbar() -> some View {
        CustomNavBar(
            title: "daemonInstall.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func explanationText() -> some View {
        HStack {
            Text(daemonSectionText())
                .textStyle(.Body.Large.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.leading)
            Spacer()
        }
        .padding(EdgeInsets(top: 16, leading: 16, bottom: 16, trailing: 16))
    }

    func firstStepText() -> some View {
        HStack {
            Text("daemonInstall.firstStep".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.leading)
            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 16))
    }

    func openSystemSettingsButton() -> some View {
        GenericButton(
            title: "daemonInstall.openSystemSettings".localizedString,
            height: 40,
            isWidthExpanded: false
        )
        .padding(.bottom, 24)
        .onTapGesture {
            SMAppService.openSystemSettingsLoginItems()
        }
    }

    func secondStepText() -> some View {
        HStack {
            Text("daemonInstall.secondStep".localizedString)
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
        if let thirdStepText = thirdStepAttributedString() {
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
        if isServing {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $isSuccessModalDisplayed,
                    configuration: successInstallActionDialogConfiguration(),
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: isSuccessModalDisplayed)
        }
    }
}

private extension DaemonInstallView {
    func daemonSectionText() -> String {
    """
    \("daemonInstall.daemonText".localizedString)
    \("daemonInstall.daemonText1".localizedString)

    \("daemonInstall.daemonText2".localizedString)
    """
    }

    func thirdStepAttributedString() -> AttributedString? {
        try? AttributedString(markdown: "\("daemonInstall.thirdStep".localizedString)  [\("daemonInstall.thirdStep.supportTeam".localizedString)](\(Constants.supportURL.rawValue))")
    }

    func successInstallActionDialogConfiguration() -> ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "checkmark",
            systemIconImageColor: NymColor.action,
            titleLocalizedString: "daemonInstall.successModal.title".localizedString,
            subtitleLocalizedString: "daemonInstall.successModal.subtitle".localizedString,
            yesLocalizedString: "daemonInstall.backToMainScreen".localizedString,
            yesAction: {
                navigateBack()
            }
        )
    }
}

// MARK: - Actions -
private extension DaemonInstallView {
    func navigateBack() {
        path = .init()
    }
}
#endif
