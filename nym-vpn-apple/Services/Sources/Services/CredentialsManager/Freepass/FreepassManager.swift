import Foundation
import Logging
#if os(iOS)
import ErrorHandler
import PathManager
#elseif os(macOS)
import GRPCManager
#endif
import NymVPNLib

@MainActor
public final class FreepassManager {
    public static let shared = FreepassManager(credentialsManager: .shared)

#if os(iOS)
    static let accountCreationTimeout: Duration = .seconds(30)
#endif

    private let credentialsManager: CredentialsManager
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    private let logger = Logger(label: "FreepassManager")
    private var queue: Task<Void, Never>?

    init(credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
    }

    /// Not cancellable, so a local account never outlives a failed redeem.
    public func redeem(code: String) async -> FreepassResult {
        let previous = queue
        let task = Task { @MainActor [unowned self] in
            await previous?.value
            switch await isAccountStored() {
            case true:
                return await applyToStoredAccount(code)
            case false:
                return await createAccount(code)
            case nil:
                return FreepassResult.failed(.unknown)
            }
        }
        queue = Task { _ = await task.value }
        return await task.value
    }
}

private extension FreepassManager {
#if os(iOS)
    typealias AccountClient = NymAccountController
#elseif os(macOS)
    typealias AccountClient = GRPCManager
#endif

    func isAccountStored() async -> Bool? {
        do {
#if os(iOS)
            let env = try credentialsManager.resolvedNetworkEnvironment()
            return try await Task {
                let dataDir = try PathManager.dataFolderURL().path()
                return try await NymVpnAccountStorage(dataDir: dataDir, environment: env).isAccountMnemonicStored()
            }.value
#elseif os(macOS)
            return try await grpcManager.isAccountStored()
#endif
        } catch {
            logger.warning("Account lookup failed: \(String(describing: type(of: error)))")
            return nil
        }
    }

#if os(iOS)
    // An account unknown to the API is registered by redeem, with the pass.
    func applyToStoredAccount(_ code: String) async -> FreepassResult {
        let result = await call { _ = try await $0.applyFreepass(code: code) }
        guard result == .failed(.accountNotRegistered) else { return result }
        return await call { try await $0.redeemFreepass(code: code) }
    }

    func createAccount(_ code: String) async -> FreepassResult {
        guard await createLocalAccount() else { return .failed(.unknown) }
        let result = await call { try await $0.redeemFreepass(code: code) }
        // A timeout or unknown failure may still have redeemed.
        if case let .failed(error) = result, error.isRejection {
            do {
                try await credentialsManager.removeCredential()
            } catch {
                logger.warning("Removing the local account failed: \(String(describing: type(of: error)))")
            }
        }
        return result
    }

    // The Rust call can't be cancelled, so it's raced against the timeout.
    func createLocalAccount() async -> Bool {
        await withCheckedContinuation { continuation in
            let outcome = CreationOutcome(continuation)
            Task { @MainActor [credentialsManager, logger] in
                do {
                    try await credentialsManager.createMnemonic()
                    outcome.finish(created: true)
                } catch {
                    logger.warning("Account creation failed: \(String(describing: type(of: error)))")
                    outcome.finish(created: false)
                }
            }
            Task { @MainActor [logger] in
                try? await Task.sleep(for: Self.accountCreationTimeout)
                if outcome.finish(created: false) {
                    logger.warning("Account creation timed out")
                }
            }
        }
    }
#elseif os(macOS)
    func applyToStoredAccount(_ code: String) async -> FreepassResult {
        await call { try await $0.redeemFreepass(code: code) }
    }

    // The daemon asks the user to confirm, so there's no timeout.
    func createAccount(_ code: String) async -> FreepassResult {
        await call { try await $0.createAccount(freepass: code) }
    }
#endif

    func call(_ body: (AccountClient) async throws -> Void) async -> FreepassResult {
        do {
#if os(iOS)
            try await credentialsManager.withController(body)
#elseif os(macOS)
            try await body(grpcManager)
#endif
            return .redeemed
        } catch {
            return .failed(FreepassError(error))
        }
    }
}

#if os(iOS)
private final class CreationOutcome: @unchecked Sendable {
    private var continuation: CheckedContinuation<Bool, Never>?

    init(_ continuation: CheckedContinuation<Bool, Never>) {
        self.continuation = continuation
    }

    @discardableResult
    func finish(created: Bool) -> Bool {
        guard let continuation else { return false }
        self.continuation = nil
        continuation.resume(returning: created)
        return true
    }
}
#endif

extension FreepassError {
#if os(iOS)
    init(_ error: Error) {
        if let reason = error as? VPNErrorReason {
            switch reason {
            case .accountStoreBusy:
                self = .vpnConnected
            case .offline, .networkConnectionError:
                self = .offline
            default:
                self = .unknown
            }
            return
        }
        guard let vpnError = error as? VpnError else {
            self = .unknown
            return
        }
        switch vpnError {
        case let .VpnApi(details: .Response(response)):
            self = .fromApi(messageId: response.messageId, message: response.message)
        case let .VpnApi(details: .StatusCode(code: code, msg: message)):
            self = code == 429 ? .rateLimited : .fromApi(messageId: nil, message: message)
        case .VpnApi, .VpnApiTimeout, .UnexpectedVpnApiResponse:
            self = .server
        case .NetworkConnectionError:
            self = .offline
        default:
            self = .unknown
        }
    }
#elseif os(macOS)
    init(_ error: Error) {
        guard let accountError = (error as? RpcError)?.accountError() else {
            self = .unknown
            return
        }
        switch accountError {
        case let .VpnApi(.Response(response)):
            self = .fromApi(messageId: response.messageId, message: response.message)
        case let .VpnApi(.StatusCode(code: code, msg: message)):
            self = code == 429 ? .rateLimited : .fromApi(messageId: nil, message: message)
        case .VpnApi, .UnexpectedVpnApiResponse:
            self = .server
        case .Offline:
            self = .offline
        default:
            self = .unknown
        }
    }
#endif
}
