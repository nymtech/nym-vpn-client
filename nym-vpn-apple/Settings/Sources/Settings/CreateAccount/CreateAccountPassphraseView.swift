import SwiftUI
import Keychain
import ImpactGenerator
import Theme
import UIComponents

public struct CreateAccountPassphraseView: View {
    @State private var messageOverlayText: String?
    @State private var displayMessageOverlay = false
    @Binding private var isAnimating: Bool
    @Binding private var isPassphraseSaved: Bool
    @Binding private var mnemonic: String?
    private var passphraseSuccessfullySavedAction: (@Sendable @MainActor () -> Void)?

    public var body: some View {
        if #available(macOS 13.3, *) {
            content
                .scrollBounceBehavior(.basedOnSize)
        } else {
            content
        }
        savedConfirmation
    }

    public init(
        isAnimating: Binding<Bool>,
        isPassphraseSaved: Binding<Bool>,
        mnemonic: Binding<String?>,
        passphraseSuccessfullySavedAction: (@Sendable @MainActor () -> Void)?
    ) {
        _isAnimating = isAnimating
        _isPassphraseSaved = isPassphraseSaved
        _mnemonic = mnemonic
        self.passphraseSuccessfullySavedAction = passphraseSuccessfullySavedAction
    }
}

private extension CreateAccountPassphraseView {
    var content: some View {
        ScrollView {
            yourPassphraseTitle
            Spacer()
                .frame(height: 16)
            passphrase
            Spacer()
                .frame(height: 16)
            exportSection
            Spacer()
                .frame(height: 40)
            howToKeepPassphraseSafe
            Spacer()
                .frame(height: 16)
            dontLoosePassphrase
            Spacer()
                .frame(height: 16)
            dontSharePassphrase
            Spacer()
        }
        .padding(.bottom, 8)
    }

    var yourPassphraseTitle: some View {
        HStack {
            Text("createAccount.yourPassphrase".localizedString)
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var passphrase: some View {
        VStack(alignment: .center, spacing: 12) {
            HStack {
                if isAnimating {
                    AnimationView(animationName: "createAccountAnimation", isAnimating: $isAnimating)
                } else {
                    if let mnemonic {
                        Text(mnemonic)
                            .foregroundStyle(NymColor.black)
                            .textStyle(.Body.Large.regular)
                    }
                }
            }
        }
        .padding(12)
        .background(NymColor.white)
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 6)
                .inset(by: 0.5)
                .stroke(NymColor.gray2, lineWidth: 1)
        )
    }

    var exportSection: some View {
        HStack(spacing: 0) {
            Spacer()
            GenericImage(systemImageName: "key.horizontal")
                .frame(width: 24, height: 24)
                .foregroundStyle(NymColor.primary)
                .onTapGesture {
                    Task {
                        await storeInKeychain()
                    }
                }
                .accessibilityAction {
                    Task {
                        await storeInKeychain()
                    }
                }

            Spacer()
                .frame(width: 16)

            GenericImage(imageName: "copy")
                .frame(width: 24, height: 24)
                .foregroundStyle(NymColor.primary)
                .onTapGesture {
                    copyToPasteboard()
                }
                .accessibilityAction {
                    copyToPasteboard()
                }
        }
        .overlay {
            if displayMessageOverlay, let messageOverlayText {
                HStack {
                    Spacer()
                    Text(messageOverlayText)
                        .padding(8)
                        .background(NymColor.elevation)
                        .foregroundColor(NymColor.gray1)
                        .cornerRadius(8)
                        .transition(.opacity)
                        .padding(.trailing, 0)
                }
                .animation(.easeInOut, value: displayMessageOverlay)
            }
        }
    }

    var howToKeepPassphraseSafe: some View {
        HStack {
            Text("createAccount.howToKeepPassphraseSafe".localizedString)
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
                .multilineTextAlignment(.leading)
            Spacer()
        }
    }

    var dontLoosePassphrase: some View {
        HStack(alignment: .top, spacing: 0) {
            GenericImage(imageName: "save")
                .frame(width: 24, height: 24)
                .padding(.trailing, 8)
                .foregroundStyle(NymColor.primary)

            VStack(alignment: .leading, spacing: 0) {
                Text("createAccount.dontLoosePassphraseTitle".localizedString)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.primary)

                Text("createAccount.dontLoosePassphraseSubtitle".localizedString)
                    .textStyle(.Body.Small.regular)
                    .foregroundStyle(NymColor.gray1)
            }
            Spacer()
        }
    }

    var dontSharePassphrase: some View {
        HStack(alignment: .top, spacing: 0) {
            GenericImage(imageName: "doNotShare")
                .frame(width: 24, height: 24)
                .padding(.trailing, 10)
                .foregroundStyle(NymColor.primary)

            VStack(alignment: .leading, spacing: 0) {
                Text("createAccount.dontShareTitle".localizedString)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.primary)

                Text("createAccount.dontShareSubtitle".localizedString)
                    .textStyle(.Body.Small.regular)
                    .foregroundStyle(NymColor.gray1)
            }
            Spacer()
        }
    }

    @ViewBuilder var savedConfirmation: some View {
        if let mnemonic, !mnemonic.isEmpty, !isAnimating {
            VStack(spacing: 0) {
                HStack(alignment: .top, spacing: 0) {
                    GenericImage(systemImageName: isPassphraseSaved ? "checkmark.square.fill" : "square")
                        .frame(width: 22, height: 22)
                        .padding(.trailing, 8)
                        .foregroundStyle(isPassphraseSaved ? NymColor.accent : NymColor.primary)
                        .transition(.scale.combined(with: .opacity))

                    savedConfirmationText
                    Spacer()
                }
                .padding(8)
                .background(Color(red: 0.57, green: 0.54, blue: 1).opacity(0.15))
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .onTapGesture {
                    isPassphraseSavedToggle()
                }
                .accessibilityAction {
                    isPassphraseSavedToggle()
                }

                if isPassphraseSaved {
                    Spacer().frame(height: 16)

                    GenericButton(title: "createAccount.continue".localizedString)
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                        .onTapGesture {
                            passphraseSuccessfullySavedAction?()
                        }
                        .accessibilityAction {
                            passphraseSuccessfullySavedAction?()
                        }
                }
            }
            .animation(.easeOut, value: isPassphraseSaved)
        }
    }

    var savedConfirmationText: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("createAccount.savedPassphrase".localizedString)
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)

            Text("createAccount.noPassphraseNoAccess".localizedString)
                .textStyle(.Body.Small.regular)
                .foregroundStyle(NymColor.primary)
        }
    }
}

// MARK: - Actions -
private extension CreateAccountPassphraseView {
    func isPassphraseSavedToggle() {
        withAnimation {
            isPassphraseSaved.toggle()
        }
    }

    func copyToPasteboard() {
        guard let mnemonic else { return }
#if os(iOS)
        UIPasteboard.general.string = mnemonic
        ImpactGenerator.shared.impact()
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(mnemonic, forType: .string)
#endif
        withAnimation {
            messageOverlayText = "settings.copiedToPasteboard".localizedString
            displayMessageOverlay = true
            shcheduleMessageOverlayDismissal()
        }
    }

    func storeInKeychain() async {
        do {
            guard let mnemonic else { return }
            try await Keychain.addInternetPassword(with: mnemonic)
        } catch {
            messageOverlayText = error.localizedDescription
            displayMessageOverlay = true
            shcheduleMessageOverlayDismissal()
            return
        }

        withAnimation {
            messageOverlayText = "settings.savedInKeychain".localizedString
            displayMessageOverlay = true
            shcheduleMessageOverlayDismissal()
        }
    }

    func shcheduleMessageOverlayDismissal() {
        Task { @MainActor in
            try? await Task.sleep(for: .seconds(3))
            displayMessageOverlay = false
        }
    }
}
