import SwiftUI
import Theme
import UIComponents

public struct CreateAccountNoPassphraseView: View {
    @Binding private var isLoading: Bool
    private var createPassphraseAction: (@Sendable @MainActor () -> Void)?

    public var body: some View {
        if #available(macOS 13.3, *) {
            content
                .scrollBounceBehavior(.basedOnSize)
        } else {
            content
        }
        createPassphraseButton
    }

    public init(isLoading: Binding<Bool>, createPassphraseAction: (@Sendable @MainActor () -> Void)?) {
        _isLoading = isLoading
        self.createPassphraseAction = createPassphraseAction
    }
}

private extension CreateAccountNoPassphraseView {
    var content: some View {
        ScrollView {
            privacyEliminatedListTitle
            Spacer()
                .frame(height: 16)
            eliminatedList
            Spacer()
                .frame(height: 40)
            insteadText
            Spacer()
                .frame(height: 24)
            enablesList
            Spacer()
                .frame(height: 24)
            whatIsPassphrase
            Spacer()
        }
        .padding(.bottom, 16)
    }

    var privacyEliminatedListTitle: some View {
        HStack {
            Text("createAccount.yourPrivacyEliminated".localizedString)
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var eliminatedList: some View {
        HStack(alignment: .center, spacing: 10) {
            listItem(with: "createAccount.thirdPartyTrust")
            listItem(with: "createAccount.emailUsage")
            listItem(with: "createAccount.passwordCreation")
        }
    }

    var insteadText: some View {
        Text("createAccount.insteadCreatePassphrase".localizedString)
            .textStyle(.Body.Large.regular)
            .foregroundStyle(NymColor.primary)
    }

    var enablesList: some View {
        HStack(alignment: .center, spacing: 10) {
            listItem(with: "createAccount.enablesSelfCustody")
            listItem(with: "createAccount.anonymousByDesign")
            listItem(with: "createAccount.actsAsLogin")
        }
    }

    var whatIsPassphrase: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack {
                GenericImage(systemImageName: "info.circle")
                    .frame(width: 16, height: 16)
                    .foregroundStyle(NymColor.primary)

                Text("createAccount.whatIsPassphrase".localizedString)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.primary)
            }

            Text("createAccount.whatIsPassphraseExplanation".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color(red: 0.57, green: 0.54, blue: 1).opacity(0.15))
        .cornerRadius(8)
    }

    var createPassphraseButton: some View {
        GenericButton(
            title: "createAccount.createPassphraseButtonTitle".localizedString,
            isLoading: $isLoading,
            systemImageNamge: "key.horizontal.fill",
            isSystemImageFlipped: true
        )
        .onTapGesture {
            createPassphraseAction?()
        }
        .accessibilityAction {
            createPassphraseAction?()
        }
    }
}

private extension CreateAccountNoPassphraseView {
    func listItem(with title: String) -> some View {
        VStack(alignment: .center, spacing: 0) {
            GenericImage(imageName: title)
                .frame(width: 28, height: 28)
                .padding(.bottom, 8)

            Text(title.localizedString)
                .multilineTextAlignment(.center)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
        .accessibilityLabel(title)
    }
}
