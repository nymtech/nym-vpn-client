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

    private let credentialsManager: CredentialsManager
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    private let logger = Logger(label: "FreepassManager")
    private var queue: Task<Void, Never>?

    init(credentialsManager: CredentialsManager) {
        self.credentialsManager = credentialsManager
    }

    /// Redeems run one at a time.
    public func redeem(code: String) async -> FreepassResult {
        let previous = queue
        let task = Task { @MainActor [unowned self] in
            await previous?.value
            switch await isAccountStored() {
            case true:
                return await redeemForStoredAccount(code)
            case false:
                return await redeemForNewAccount(code)
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

    // Core registers an account the API doesn't know yet.
    func redeemForStoredAccount(_ code: String) async -> FreepassResult {
#if os(iOS)
        await call { try await $0.applyFreepass(code: code) }
#elseif os(macOS)
        await call { try await $0.redeemFreepass(code: code) }
#endif
    }

    // A rejected code leaves no account.
    func redeemForNewAccount(_ code: String) async -> FreepassResult {
        await call { try await $0.createAccount(freepass: code) }
    }

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
        case .FreepassInvalid:
            self = .invalidCode
        case .FreepassAlreadyRedeemed:
            self = .alreadyRedeemed
        case .RecurringSubscriptionActive:
            self = .subscriptionActive
        case .RateLimited:
            self = .rateLimited
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
        case .FreepassInvalid:
            self = .invalidCode
        case .FreepassAlreadyRedeemed:
            self = .alreadyRedeemed
        case .RecurringSubscriptionActive:
            self = .subscriptionActive
        case .RateLimited:
            self = .rateLimited
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
