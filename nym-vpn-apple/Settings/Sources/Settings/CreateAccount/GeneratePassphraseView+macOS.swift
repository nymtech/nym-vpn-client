#if os(macOS)
import Constants

extension GeneratePassphraseView {
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
                alertTitle = error.localizedDescription
                isAlertDisplayed = true
            }
            return
        }
        didRegisterAccount = true
    }

    func selectPlanAction() {
        isPurchasing = true
        Task {
            await autologinState.perform(kind: .autologinRenew, using: credentialsManager)
            isPurchasing = false
        }
    }
}
#endif
