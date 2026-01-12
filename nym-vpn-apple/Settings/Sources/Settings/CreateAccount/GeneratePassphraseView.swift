import SwiftUI
import AppSettings
import CredentialsManager
#if os(iOS)
import ImpactGenerator
import NymVPNLib
import ErrorHandler
#endif
import Theme
import UIComponents

public struct GeneratePassphraseView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @Binding private var path: NavigationPath
    @State private var didFinishAnimatingText = false
    @State private var alertTitle: String = ""
    @State private var isAlertDisplayed = false
    @State private var didRegisterAccount = false
    @State private var currentStep = 1

    public var body: some View {
        VStack(spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)

            StepView(stepCount: 4, currentStep: $currentStep)
            Spacer()

            dotsAnimationView
            Spacer()
                .frame(height: 16)

            animatingTextView

            Spacer()
        }
        .frame(maxWidth: MagicNumbers.moreMaxWidth)
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .task {
            await generateAndRegisterMnemonic()
        }
        .alert(alertTitle, isPresented: $isAlertDisplayed) {
            Button("retry".localizedString, role: .cancel) {
                Task {
                    await generateAndRegisterMnemonic()
                }
            }
        }
        .onChange(of: didFinishAnimatingText) { _, _ in
            Task {
                try? await Task.sleep(for: .seconds(2))
                navigateToPlanSelectIfNeeded()
            }
        }
        .onChange(of: didRegisterAccount) { _, _ in
            Task {
                try? await Task.sleep(for: .seconds(2))
                navigateToPlanSelectIfNeeded()
            }
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Views -
private extension GeneratePassphraseView {
    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var dotsAnimationView: some View {
        WaveDotsView()
    }

    var animatingTextView: some View {
        SwitchingTitlesView(
            pairs: [
                ("generatePassphrase.title1".localizedString, "generatePassphrase.subtitle1".localizedString),
                ("generatePassphrase.title2".localizedString, "generatePassphrase.subtitle2".localizedString),
                ("generatePassphrase.title3".localizedString, "generatePassphrase.subtitle3".localizedString)
            ],
            didFinishAnimating: $didFinishAnimatingText,
            timerDidTick: {
                currentStep += 1
            }
        )
    }
}

// MARK: - Actions -
private extension GeneratePassphraseView {
    func generateAndRegisterMnemonic() async {
        do {
            if appSettings.isCredentialImported {
                try await credentialsManager.registerAccount()
            } else {
                try await credentialsManager.createMnemonic()
                try await credentialsManager.registerAccount()
            }
        } catch {
            Task { @MainActor in
#if os(iOS)
                if let lastVPNError = error as? VpnError {
                    alertTitle = VPNErrorReason(with: lastVPNError).errorDescription ?? ""
                } else {
                    alertTitle = error.localizedDescription
                }
                isAlertDisplayed = true
#endif
            }
            return
        }
        didRegisterAccount = true
    }

    func navigateToPlanSelectIfNeeded() {
        Task { @MainActor in
            guard didFinishAnimatingText, didRegisterAccount else { return }
            path.append(SettingLink.planPurchase(shouldDisplayBackButton: false))
        }
    }
}
