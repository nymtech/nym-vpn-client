import SwiftUI
import ImpactGenerator
import UIComponents
import Theme

public struct WhatIsPassphraseView: View {
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @Binding private var isDisplayed: Bool

    public init(isDisplayed: Binding<Bool>) {
        _isDisplayed = isDisplayed
    }

    public var body: some View {
        ZStack {
            backgroundRectangle

            HStack {
                Spacer()
                    .frame(width: 40)

                content
                    .padding(24)
                    .background(NymColor.elevation)
                    .cornerRadius(16)

                Spacer()
                    .frame(width: 40)
            }
            .frame(maxWidth: MagicNumbers.moreMaxWidth)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .edgesIgnoringSafeArea(.all)
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

    var backgroundRectangle: some View {
        Rectangle()
            .foregroundColor(.black)
            .opacity(0.3)
            .background(Color.clear)
            .contentShape(Rectangle())
    }

    var title: some View {
        Text("whatIsPasshphrase.title".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)
    }

    var subtitle: some View {
        Text(subtitleAttributtedString ?? "")
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
    }

    var subtitleAttributtedString: AttributedString? {
        let first = AttributedString("whatIsPassphrase.subtitle1".localizedString)
        var second = AttributedString("whatIsPassphrase.keepItSafe".localizedString)
        second.font = NymTextStyle.Body.Medium.bold.nymFont.font
        let third = AttributedString("whatIsPassphrase.subtitle2".localizedString)
        return first + AttributedString("\n\n") + AttributedString("⚠️ ") + second + AttributedString(": ") + third
    }

    var removedTitle: some View {
        Text("whatIsPassphrase.removedTitle".localizedString)
            .textStyle(.Body.Medium.bold)
            .foregroundStyle(NymColor.primary)
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
            .textStyle(.Body.Medium.bold)
            .foregroundStyle(NymColor.primary)
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
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
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
