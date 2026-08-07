import Foundation
import Logging
#if os(iOS)
import ErrorHandler
import NymVPNLib
#endif

enum AccountRegistrationSupport {
#if os(iOS)
    static func isAccountStoreBusyFailure(_ error: Error) -> Bool {
        if let reason = error as? VPNErrorReason, case .accountStoreBusy = reason {
            return true
        }
        return false
    }

    static func mapToVPNErrorReason(_ error: Error) -> Error {
        if error is VPNErrorReason {
            return error
        }
        if let vpnError = error as? VpnError {
            return VPNErrorReason(with: vpnError)
        }
        return error
    }

    static func environmentForCredentialImport(
        isRegistrationInFlight: Bool,
        registrationCapturedEnvironment: NymEnvironment?,
        liveNetworkEnv: NymEnvironment?
    ) -> NymEnvironment? {
        if isRegistrationInFlight, let registrationCapturedEnvironment {
            return registrationCapturedEnvironment
        }
        return liveNetworkEnv
    }
#endif
}

#if os(iOS)
extension AccountRegistrationSupport {
    static let accountStoreBusyRetryDelays: [Duration] = [
        .milliseconds(200),
        .milliseconds(500),
        .seconds(1),
        .seconds(2)
    ]

    static func withAccountStoreRetry<T>(
        operation: String,
        logger: Logger,
        _ body: () async throws -> T
    ) async throws -> T {
        var lastError: Error?
        for (attempt, delay) in accountStoreBusyRetryDelays.enumerated() {
            if attempt > 0 {
                try await Task.sleep(for: delay)
            }
            do {
                return try await body()
            } catch {
                let mapped = mapToVPNErrorReason(error)
                guard isAccountStoreBusyFailure(mapped),
                      attempt < accountStoreBusyRetryDelays.count - 1 else {
                    throw mapped
                }
                logger.debug("\(operation) account store busy, retry \(attempt + 1)")
                lastError = mapped
            }
        }
        throw lastError ?? AccountRegistrationFailure.accountStoreBusyRetryExhausted
    }
}

private enum AccountRegistrationFailure: Error {
    case accountStoreBusyRetryExhausted
}
#endif
