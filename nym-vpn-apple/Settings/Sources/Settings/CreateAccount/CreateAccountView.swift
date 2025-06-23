import SwiftUI
import CredentialsManager
#if os(iOS)
import ImpactGenerator
import MixnetLibrary
import ErrorHandler
#endif
import UIComponents
import Theme

public struct CreateAccountView: View {
    @Binding private var path: NavigationPath
    @State private var isAnimating = false
    @State private var isLoading = false
    @State private var isDisplayingAlert = false
    @State private var isPassphraseSaved = false
    @State private var alertTitle = ""
    @State private var mnemonic: String?
    @EnvironmentObject private var credentialsManager: CredentialsManager

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
                .frame(height: 40)
            HStack {
                VStack(alignment: .leading, spacing: 0) {
                    StepView(stepCount: 3, currentStep: 1)
                    Spacer()
                        .frame(height: 40)
                    createAccountTitle
                    Spacer()
                        .frame(height: 24)
                    if mnemonic == nil {
                        CreateAccountNoPassphraseView(isLoading: $isLoading) {
                            createPassphraseAction()
                        }
                    } else {
                        CreateAccountPassphraseView(
                            isAnimating: $isAnimating,
                            isPassphraseSaved: $isPassphraseSaved,
                            mnemonic: $mnemonic
                        ) {
                            navigateToSuccessAccountCreated()
                        }
                    }
                }
                Spacer()
            }
            .padding(.horizontal, 16)
            Spacer()
                .frame(height: 16)
        }
        .alert(alertTitle, isPresented: $isDisplayingAlert) {
            Button("ok".localizedString, role: .cancel) {}
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Initial Section -
private extension CreateAccountView {
    var navbar: some View {
        CustomNavBar(
            useElevationBackground: true,
            leftButton: mnemonic == nil ? CustomNavBarButton(type: .back, action: { navigateBack() }) : nil
        )
    }

    var createAccountTitle: some View {
        Text("createAccount.createAnAccount".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(.primary)
    }
}
// MARK: - Helpers -
private extension CreateAccountView {
    func clearMnemonic() {
        mnemonic = nil
    }
}

// MARK: - Actions -
private extension CreateAccountView {
    func navigateBack() {
        clearMnemonic()
        if mnemonic == nil {
            if !path.isEmpty { path.removeLast() }
        } else {
            path = .init()
        }
    }

    func navigateToSuccessAccountCreated() {
        clearMnemonic()
        path.append(SettingLink.createAccountSuccess)
    }

    func createPassphraseAction() {
        guard !isLoading else { return }
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        isAnimating = true
        Task {
            defer {
                isLoading = false
            }
            do {
                isLoading = true
                let result = try await credentialsManager.registerAccount()
                Task { @MainActor in
                    mnemonic = result.mnemonic
                }
            } catch {
                Task { @MainActor in
#if os(iOS)
                    if let lastVPNError = error as? VpnError {
                        alertTitle = VPNErrorReason(with: lastVPNError).errorDescription ?? ""
                    } else {
                        alertTitle = error.localizedDescription
                    }
                    isDisplayingAlert = true
#endif
                }
            }
        }
    }
}
