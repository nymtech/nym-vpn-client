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
            do {
                guard let result = try await credentialsManager.autologin(kind: .autologinRenew) else {
                    isPurchasing = false
                    return
                }
                isPurchasing = false
                pinCode = result.pinCode
                autologinURL = result.url
                isPinCodeDisplayed = true
            } catch is CancellationError {
                isPurchasing = false
            } catch {
                isPurchasing = false
                autologinErrorMessage = error.localizedDescription
                isAutologinError = true
            }
        }
    }
}
#endif
