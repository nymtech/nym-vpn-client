import Foundation
import Logging
#if os(iOS)
import ErrorHandler
import NymVPNLib
#endif

enum AccountRegistrationSupport {
    private static let credentialStoreLockDetail = "credential store lock"

#if os(iOS)
    static func isCredentialStoreLockFailure(_ error: Error) -> Bool {
        if let reason = error as? VPNErrorReason {
            if case .accountStoreBusy = reason { return true }
            if case let .storage(details) = reason,
               details.localizedCaseInsensitiveContains(credentialStoreLockDetail) {
                return true
            }
        }
        if let vpnError = error as? VpnError {
            if vpnError == .AccountStoreBusy { return true }
            if case let .Storage(details: details) = vpnError,
               details.localizedCaseInsensitiveContains(credentialStoreLockDetail) {
                return true
            }
        }
        return false
    }

    static func mapToVPNErrorReason(_ error: Error) -> Error {
        if error is VPNErrorReason { return error }
        if let vpnError = error as? VpnError {
            return VPNErrorReason(with: vpnError)
        }
        return error
    }

    static func userFacingDescription(for error: Error) -> String {
        if let reason = error as? VPNErrorReason {
            return reason.localizedDescription
        }
        if let vpnError = error as? VpnError {
            return VPNErrorReason(with: vpnError).localizedDescription
        }
        return error.localizedDescription
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
    static let storeLockRetryDelays: [Duration] = [
        .milliseconds(200),
        .milliseconds(500),
        .seconds(1),
        .seconds(2)
    ]

    static func withCredentialStoreRetry<T>(
        operation: String,
        logger: Logger,
        _ body: () async throws -> T
    ) async throws -> T {
        var lastError: Error?
        for (attempt, delay) in storeLockRetryDelays.enumerated() {
            if attempt > 0 {
                try await Task.sleep(for: delay)
            }
            do {
                return try await body()
            } catch {
                let mapped = mapToVPNErrorReason(error)
                guard isCredentialStoreLockFailure(mapped),
                      attempt < storeLockRetryDelays.count - 1 else {
                    throw mapped
                }
                logger.debug("\(operation) credential store lock busy, retry \(attempt + 1)")
                lastError = mapped
            }
        }
        throw lastError ?? AccountRegistrationFailure.storeLockRetryExhausted
    }
}

private enum AccountRegistrationFailure: Error {
    case storeLockRetryExhausted
}
#endif
