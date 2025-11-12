import SwiftUI
import AppSettings
import Device
import BiometricAuthenticator
import CredentialsManager
import ImpactGenerator
import Keychain
import MessageModels
import Theme
import UIComponents

public struct PassphraseView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @Binding private var path: NavigationPath
    @State private var isErrorDisplayed = false
    @State private var isSnackbarDisplayed = false
    @State private var isInfoModalDisplayed = false
    @State private var mnemonic: String?
    @State private var snackbarMessage: String?
    @State private var errorMessage = ""

    private var words: [String] {
        guard let mnemonic else { return [] }
        return mnemonic.components(separatedBy: .whitespacesAndNewlines)
    }

    private var columns: [[String]] {
        (0..<3).map { column in
            let start = column * 8
            let end = min(start + 8, words.count)
            return Array(words[start..<end])
        }
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)
                VStack(spacing: 0) {
                    title
                    Spacer()
                        .frame(height: 16)
                    subtitle
                    Spacer()
                        .frame(height: 24)
                    passphraseSquare
                    Spacer()
                        .frame(height: 24)
                    passphraseActionsRow
                    Spacer()
                        .frame(height: 24)
                    exclaimerText
                    Spacer()
                        .frame(minHeight: 0)
                    savedConfirmationSection
                }
                .frame(maxWidth: MagicNumbers.moreMaxWidth, maxHeight: .infinity)
                .padding(.horizontal, 16)
        }
        .alert(errorMessage, isPresented: $isErrorDisplayed) {
            Button("ok".localizedString, role: .cancel) {}
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .snackbar(
            isDisplayed: $isSnackbarDisplayed,
            message: SnackBarMessage(text: snackbarMessage ?? "", style: .info)
        )
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .overlay {
            if isInfoModalDisplayed {
                WhatIsPassphraseView(isDisplayed: $isInfoModalDisplayed)
            }
        }
        .onDisappear {
            mnemonic = nil
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension PassphraseView {
    var navbar: some View {
        CustomNavBar(
            title: "settings.passphrase".localizedString,
            useElevationBackground: true,
            isLogoImageHidden: true,
            leftButton: CustomNavBarButton(
                type: .back,
                action: {
                    impactGenerator.softImpact()
                    navigateBack()
                }
            ),
            rightButton: CustomNavBarButton(
                type: .info,
                action: {
                    impactGenerator.softImpact()
                    isInfoModalDisplayed.toggle()
                }
            )
        )
    }

    var title: some View {
        HStack(spacing: 0) {
            Text("passphrase.yourPassphrase".localizedString)
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }
    }

    var subtitle: some View {
        HStack {
            Text("passphrase.masterPassphrase".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .multilineTextAlignment(.leading)
            Spacer()
        }
    }

    var passphraseSquare: some View {
        VStack(alignment: .center, spacing: 24) {
            if let mnemonic, !mnemonic.isEmpty {
                passphrase
            } else {
                showPassphraseButton
            }
        }
        .frame(maxWidth: .infinity, minHeight: 290)
        .background(NymColor.elevation)
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(NymColor.gray2, lineWidth: 1)
            )
    }

    var showPassphraseButton: some View {
        HStack(spacing: 0) {
            GenericImage(systemImageName: "eye.fill")
                .frame(width: 20, height: 20)
            Spacer()
                .frame(width: 8)
            Text("passphrase.showMyPassphrase".localizedString)
                .textStyle(.Headline.Small.regular)
                .foregroundStyle(NymColor.primary)
        }
        .padding(.horizontal, 24)
        .padding(.vertical, 10)
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(NymColor.primary, lineWidth: 1)
        )
        .onTapGesture {
            showPassphraseDidTap()
        }
        .accessibilityAction {
            showPassphraseDidTap()
        }
    }

    var passphrase: some View {
        HStack(spacing: 0) {
            ForEach(0..<columns.count, id: \.self) { column in
                VStack(alignment: .leading, spacing: 16) {
                    ForEach(Array(columns[column].enumerated()), id: \.offset) { row, word in
                        let number = column * 8 + row + 1
                        HStack(spacing: 4) {
                            Text("\(number).")
                                .textStyle(.Body.Medium.regular)
                                .foregroundStyle(NymColor.gray1)
                                .monospacedDigit()
                                .frame(width: 30, alignment: .trailing)
                            Text("\(word)")
                                .textStyle(.Body.Medium.regular)
                                .foregroundStyle(NymColor.primary)
                            Spacer()
                        }
                    }
                }
            }
        }
        .padding(.horizontal, 8)
    }

    @ViewBuilder var passphraseActionsRow: some View {
        if let mnemonic, !mnemonic.isEmpty {
            HStack(spacing: 0) {
                Spacer()
                copyButton
                Spacer()
                separatorView
                Spacer()
                keychainButton
                Spacer()
            }
        }
    }

    var copyButton: some View {
        Text("passphrase.copy".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.action)
            .frame(maxWidth: .infinity)
            .onTapGesture {
                copyToPasteboard()
            }
            .accessibilityAction {
                copyToPasteboard()
            }
    }

    var separatorView: some View {
        Rectangle()
            .frame(width: 1, height: 20)
            .background(NymColor.gray2)
    }

    var keychainButton: some View {
        Text("passphrase.saveToKeychain".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.action)
            .frame(maxWidth: .infinity)
            .onTapGesture {
                storeInKeychain()
            }
            .accessibilityAction {
                storeInKeychain()
            }
    }

    @ViewBuilder var exclaimerText: some View {
        if let mnemonic, !mnemonic.isEmpty {
            HStack(spacing: 0) {
                VStack(alignment: .leading) {
                    Text("passphrase.loseTitle".localizedString)
                        .foregroundStyle(NymColor.primary)
                        .textStyle(.Body.Medium.bold)
                        .multilineTextAlignment(.leading)
                    Spacer()
                        .frame(height: 8)
                    Text("passohrase.loseSubtitle".localizedString)
                        .foregroundStyle(NymColor.primary)
                        .textStyle(.Body.Medium.regular)
                        .multilineTextAlignment(.leading)
                }
                Spacer()
            }
        }
    }

    @ViewBuilder var savedConfirmationSection: some View {
        if let mnemonic, !mnemonic.isEmpty {
            VStack(spacing: 0) {
                HStack(alignment: .top, spacing: 0) {
                    GenericImage(systemImageName: appSettings.isPassphraseStored ? "checkmark.square.fill" : "square")
                        .frame(width: 22, height: 22)
                        .padding(EdgeInsets(top: 2, leading: 0, bottom: 0, trailing: 8))
                        .foregroundStyle(NymColor.primary)
                        .transition(.scale.combined(with: .opacity))

                    savedConfirmationText
                    Spacer()
                }
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .onTapGesture {
                    withAnimation(.easeOut) {
                        toggleIsPassphraseSaved()
                    }
                }
                .accessibilityAction {
                    withAnimation(.easeOut) {
                        toggleIsPassphraseSaved()
                    }
                }

                savedConfirmationButton
            }
        }
    }

    var savedConfirmationText: some View {
        Text("passphrase.iHaveSaved".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.leading)
            .fixedSize(horizontal: false, vertical: true)
    }

    @ViewBuilder var savedConfirmationButton: some View {
        if appSettings.isPassphraseStored {
            Spacer()
                .frame(height: 16)

            GenericButton(title: "passphrase.continue".localizedString)
                .transition(.move(edge: .bottom).combined(with: .opacity))
                .onTapGesture {
                    navigateBack()
                }
                .accessibilityAction {
                    navigateBack()
                }
        }
    }
}

private extension PassphraseView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func showPassphraseDidTap() {
        impactGenerator.softImpact()
        authenticate()
    }

    func authenticate() {
        let reason: String
        switch BiometricAuthenticator.availableBiometric() {
        case .faceID:
            reason = "passphrase.faceID".localizedString
        case .touchID:
            reason = "passphrase.touchID".localizedString
        case .opticID:
            reason = "passphrase.opticID".localizedString
        case .none:
            if Device.isMacOS {
                reason = "passphrase.password".localizedString
            } else {
                reason = "passphrase.passcode".localizedString
            }
        }

        Task {
            do {
                try await BiometricAuthenticator.authenticate(reason: reason)
                mnemonic = try await credentialsManager.mnemonic()
            } catch {
                errorMessage = error.localizedDescription
                isErrorDisplayed = true
            }
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
            guard !isSnackbarDisplayed else { return }
            snackbarMessage = "settings.copiedToPasteboard".localizedString
            isSnackbarDisplayed = true
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                isSnackbarDisplayed = false
            }
        }
    }

    func storeInKeychain() {
        Task {
#if os(iOS)
            ImpactGenerator.shared.impact()
#endif
            guard let mnemonic else { return }
            do {
                try await Keychain.addInternetPassword(with: mnemonic)
            } catch {
                errorMessage = error.localizedDescription
                isErrorDisplayed = true
            }
        }
    }

    func toggleIsPassphraseSaved() {
        appSettings.isPassphraseStored.toggle()
    }
}
