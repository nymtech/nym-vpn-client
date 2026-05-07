import SwiftUI
import AppSettings
import ImpactGenerator
import UIComponents
import Theme

public struct WhatIsPassphraseView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @Binding private var isDisplayed: Bool

    public init(isDisplayed: Binding<Bool>) {
        _isDisplayed = isDisplayed
    }

    public var body: some View {
        ModalOverlayView(
            isDisplayed: $isDisplayed,
            dismissOnOverlayTap: false,
            horizontalPadding: appSettings.isSmallScreen ? 16 : 40
        ) {
            content
                .padding(24)
        }
    }
}

private extension WhatIsPassphraseView {
    var content: some View {
        VStack(spacing: 0) {
            title
            Spacer()
                .frame(height: 16)
            subtitle
            Spacer()
                .frame(height: 16)
            removedTitle
            Spacer()
                .frame(height: 24)
            eliminatedList
            Spacer()
                .frame(height: 24)
            passhpraseText
            Spacer()
                .frame(height: 24)
            enabledList
            Spacer()
                .frame(height: 24)
            doneButton
        }
    }

    var title: some View {
        Text("whatIsPasshphrase.title".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
    }

    var subtitle: some View {
        Text(subtitleAttributtedString ?? "")
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var subtitleAttributtedString: AttributedString? {
        let first = AttributedString("whatIsPassphrase.subtitle1".localizedString)
        var second = AttributedString("whatIsPassphrase.keepItSafe".localizedString)
        second.font = Nym.TextStyle.bodyDefaultBold.font
        let third = AttributedString("whatIsPassphrase.subtitle2".localizedString)
        return first + AttributedString("\n\n") + AttributedString("⚠️ ") + second + AttributedString(": ") + third
    }

    var removedTitle: some View {
        Text("whatIsPassphrase.removedTitle".localizedString)
            .nymTextStyle(.bodyDefaultBold)
            .foregroundStyle(Color.Nym.textPrimary)
    }

    var eliminatedList: some View {
        HStack(alignment: .center, spacing: 10) {
            listItem(with: "whatIsPassphrase.thirdPartyTrust")
            listItem(with: "whatIsPassphrase.emailUsage")
            listItem(with: "whatIsPassphrase.passwordCreation")
        }
    }

    var passhpraseText: some View {
        Text("whatIsPassphrase.24wordPassphrase".localizedString)
            .nymTextStyle(.bodyDefaultBold)
            .foregroundStyle(Color.Nym.textPrimary)
    }

    var enabledList: some View {
        HStack(alignment: .center, spacing: 10) {
            listItem(with: "whatIsPassphrase.enablesSelfCustody")
            listItem(with: "whatIsPassphrase.anonymousByDesign")
            listItem(with: "whatIsPassphrase.actsAsLogin")
        }
    }

    var doneButton: some View {
        GenericButton(title: "done".localizedString, style: .primaryBorderOnly)
            .onTapGesture {
                closeView()
            }
            .accessibilityAction {
                closeView()
            }
    }
}

private extension WhatIsPassphraseView {
    func listItem(with title: String) -> some View {
        VStack(alignment: .center, spacing: 0) {
            GenericImage(imageName: title)
                .frame(width: 28, height: 28)
                .padding(.bottom, 8)

            Text(title.localizedString)
                .multilineTextAlignment(.center)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textPrimary)
        }
        .accessibilityLabel(title)
        .frame(maxWidth: .infinity)
    }
}

private extension WhatIsPassphraseView {
    func closeView() {
        impactGenerator.softImpact()
        isDisplayed = false
    }
}
