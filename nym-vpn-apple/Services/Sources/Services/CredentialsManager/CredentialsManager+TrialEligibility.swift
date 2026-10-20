#if os(iOS)
import NymVPNLib
import PathManager
import PurchasesManager

extension CredentialsManager: NymTrialEligibilityProviding {
    public func isNymTrialEligible() async -> Bool {
        do {
            let environment = try resolvedRegistrationEnvironment()
            let isAccountStored = try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                return try await NymVpnAccountStorage(
                    dataDir: dataDir,
                    environment: environment
                ).isAccountMnemonicStored()
            }.value
            // No stored account means a fresh user with no history, like the API's "account not found".
            guard isAccountStored else {
                logger.debug("Trial eligibility: eligible, no stored account")
                return true
            }
            let isEligible = try await withController { controller in
                try await controller.isTrialEligible()
            }
            logger.debug("Trial eligibility: eligible=\(isEligible)")
            return isEligible
        } catch {
            logger.debug("Trial eligibility: not eligible, lookup failed")
            return false
        }
    }

    public func withIntroOfferEligibilityLookups<T: Sendable>(
        _ body: @Sendable (IntroOfferEligibilityLookups) async -> T
    ) async throws -> T {
        // One controller for both calls, so the second does not wait on the first one's shutdown.
        try await withController { controller in
            await body(
                IntroOfferEligibilityLookups(
                    isTrialEligible: {
                        try await controller.isTrialEligible()
                    },
                    checkAppleIntroOfferEligibility: { productId, transactionId in
                        try await controller.checkAppleIntroOfferEligibility(
                            productId: productId,
                            transactionId: transactionId
                        )
                    }
                )
            )
        }
    }
}
#endif
