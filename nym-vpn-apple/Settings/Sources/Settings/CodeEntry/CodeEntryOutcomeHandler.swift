import Foundation
import AccountPrefetchGates
import CredentialsManager
import SnackbarManager
import Theme

@MainActor
public enum CodeEntryOutcomeHandler {
    public static func handle(
        _ outcome: CodeEntryOutcome,
        credentialsManager: CredentialsManager,
        sendSessionEvent: @escaping (SessionEvent) -> Void
    ) async {
        switch outcome {
        case .freepassRedeemed:
            SnackbarManager.shared.enqueue(
                SnackbarItem(style: .confirmation, title: "freepass.activated".localizedString)
            )
            if credentialsManager.isValidCredentialImported {
                await credentialsManager.updateAccountSummary(force: true, untilActive: true)
            } else {
                await startLoginProcessing(credentialsManager: credentialsManager, sendSessionEvent: sendSessionEvent)
            }
        case .pairingSignedIn:
            await startLoginProcessing(credentialsManager: credentialsManager, sendSessionEvent: sendSessionEvent)
        case .cancelled:
            break
        }
    }

    private static func startLoginProcessing(
        credentialsManager: CredentialsManager,
        sendSessionEvent: @escaping (SessionEvent) -> Void
    ) async {
        // Sent before the account is noticed, so it isn't taken for an outside sign-in.
        sendSessionEvent(.authWillBegin(flow: .login, completesOnCredentialImport: false))
        let outcome = await AuthCompletionOutcomeResolver.resolveAfterLoginRegistration(
            isAccountActive: { credentialsManager.isAccountActive() },
            updateAccountSummary: { await credentialsManager.updateAccountSummary(force: true, untilActive: false) }
        )
        sendSessionEvent(.authCompleted(outcome: outcome, flow: .login))
    }
}
