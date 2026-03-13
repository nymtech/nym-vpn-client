import Combine
import Logging
import SwiftUI
import Foundation
import AppSettings
import AppVersionProvider
import ConnectionTypes
import ConfigurationManager
import Constants
import ErrorReason
#if os(iOS)
import ErrorHandler
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import PathManager

@MainActor public final class CredentialsManager: ObservableObject {
    private let logger = Logger(label: "CredentialsManager")
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    private let appSettings = AppSettings.shared
    private let configurationManager = ConfigurationManager.shared

#if os(iOS)
    var deeplinks: NymDeeplinks?
#endif
    private var cancellables = Set<AnyCancellable>()

    public static let shared = CredentialsManager()

    public var deviceIdentifier: String?
    @Published public var accountIdentifier: String?
    @Published public var didReceiveAccountLinkCallback = false
    @Published public var accountSummary: AccountSummary?

    public var isValidCredentialImported: Bool {
        appSettings.isCredentialImported
    }

    public var accountToken: String? {
        appSettings.accountToken
    }

    private init() {}

    public func setup() {
#if os(iOS)
        checkCredentialImport()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif
    }

    public func add(credential: String) async throws {
        try await Task {
            do {
#if os(iOS)
                try await NymVpnAccountStorage(
                    dataDir: PathManager.dataFolderURL().path(),
                    environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
                ).login(request: .vpn(mnemonic: credential))
#elseif os(macOS)
                try await grpcManager.storeAccount(with: .vpn(mnemonic: credential))
#endif
                checkCredentialImport()
            } catch {
#if os(iOS)
                if let vpnError = error as? VpnError {
                    throw VPNErrorReason(with: vpnError)
                } else {
                    throw error
                }
#elseif os(macOS)
                throw error
#endif
            }
        }.value
    }

    public func createMnemonic() async throws {
#if os(iOS)
        try await Task {
            try await NymVpnAccountStorage(
                dataDir: PathManager.dataFolderURL().path(),
                environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
            ).createAccount()
            Task { @MainActor in
                checkCredentialImport()
            }
        }.value
#endif
    }

    public func mnemonic() async throws -> String {
#if os(iOS)
        try await Task {
            try await NymVpnAccountStorage(
                dataDir: PathManager.dataFolderURL().path(),
                environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
            ).getStoredMnemonic()
        }.value
#elseif os(macOS)
        // TODO: add missing grpc
        return ""
#endif
    }

    public func registerAccount() async throws {
#if os(iOS)
        do {
            let result = try await NymVpnAccountStorage(
                dataDir: PathManager.dataFolderURL().path(),
                environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
            ).registerAccount()
            Task { @MainActor in
                appSettings.accountToken = result.accountToken
                checkCredentialImport()
            }
        }
#endif
    }

    public func removeCredential() async throws {
        try await Task {
            do {
#if os(iOS)
                try await NymVpnAccountStorage(
                    dataDir: PathManager.dataFolderURL().path(),
                    environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
                ).forgetAccount()
#elseif os(macOS)
                try await grpcManager.forgetAccount()
#endif
                checkCredentialImport()
            } catch {
                // TODO: need modal for alerts
                throw error
            }
            Task { @MainActor in
                appSettings.accountToken = nil
                accountSummary = nil
            }
        }.value
    }

    public func privyLogin(isLink: Bool = false) async throws -> String? {
        didReceiveAccountLinkCallback = false
        let locale = Locale.current.language.languageCode?.identifier.lowercased() ?? "en"
        let name = "default"
#if os(iOS)
        guard let networkEnv = configurationManager.networkEnv else { return nil }

        deeplinks = NymDeeplinks(networkEnv: networkEnv)
        return try await deeplinks?.getDeeplink(
            params: .init(
                client: .mobile,
                locale: locale,
                kind: isLink ? .privyLink : .privy,
                name: name,
                redirectPath: nil
            )
        )
#elseif os(macOS)
        return try await grpcManager.privyLogin(locale: locale, name: name, isLink: isLink)
#endif
    }

    public func privyLoginStore(callbackURLString: String) async throws {
#if os(iOS)
        guard let deeplinks, let networkEnv = configurationManager.networkEnv else { return }
        let mnemonic = try await deeplinks.deriveMnemonic(deeplinkCallbackUrl: callbackURLString)
        try await NymVpnAccountStorage(
            dataDir: PathManager.dataFolderURL().path(),
            environment: networkEnv
        ).loginWithDeeplinkMnemonic(deeplinkMnemonic: mnemonic)
        self.deeplinks = nil
        checkCredentialImport()
#elseif os(macOS)
        try await grpcManager.storePrivyAccount(with: callbackURLString)
        checkCredentialImport()
#endif
        didReceiveAccountLinkCallback = true
    }

    /// Fetches account summary from API if current accountSummary.validUntilDate does not exist or is in past,
    /// stores value and returns true if validUntilDate is in the future
    /// - Returns: Bool
    public func isAccountValid() async -> Bool {
        if isAccountSubscriptionDateValid() {
            return true
        } else {
            await updateAccountSummary()
            return isAccountSubscriptionDateValid()
        }
    }

    public func updateAccountSummary() async {
#if os(iOS)
        guard let summary = try? await NymVpnAccountStorage(
            dataDir: PathManager.dataFolderURL().path(),
            environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
        ).getAccountSummary()
        else {
            return
        }

        accountSummary = AccountSummary(
            validUntilTimeInterval: summary.subscriptionValidUntil,
            trafficUsedGb: summary.trafficUsedGb,
            trafficLimitGb: summary.trafficLimitGb,
            trafficResetTimeInterval: summary.trafficResetTime,
            accountAddress: summary.accountAddr,
            cannonicalAccountAddress: summary.canonicalAccountAddr,
            accountAuthMethod: summary.authMethods.map { AccountAuthMethod(vpnAccountMethod: $0) },
            isLinked: summary.isLinked(),
            isActive: summary.isSubscriptionActive(),
            isAutoRenewEnabled: summary.isRecurring,
            subscriptionKind: summary.subscriptionKind.map { VpnSubscriptionKind(from: $0) }
        )

#elseif os(macOS)
        accountSummary = try? await grpcManager.accountSummary()
#endif
        resetExpiryDismissalsIfNeeded()
    }

    private func resetExpiryDismissalsIfNeeded() {
        guard let accountSummary,
              accountSummary.isActive,
              !accountSummary.isExpiringSoon,
              !accountSummary.isExpiringWarning
        else {
            return
        }
        appSettings.expiryWarningDismissedAt = 0
        appSettings.expirySoonDismissedAt = 0
    }
}

private extension CredentialsManager {
    func setupGRPCManagerObservers() {
#if os(macOS)
        grpcManager.$errorReason.sink { [weak self] error in
            guard let self,
                  let errorReason = error as? ErrorReason,
                  errorReason == .noAccountStored
            else {
                return
            }
            Task { @MainActor in
                self.appSettings.isCredentialImported = false
            }
        }
        .store(in: &cancellables)

        grpcManager.$isServing.sink { [weak self] isServing in
            guard let self, isServing else { return }
            checkCredentialImport()
        }
        .store(in: &cancellables)
#endif
    }
}

private extension CredentialsManager {
    /// Checks if accountSummary.validUntilDate is in the future
    /// - Returns: Bool
    func isAccountSubscriptionDateValid() -> Bool {
        guard let validUntilDate = accountSummary?.validUntilDate,
              validUntilDate > Date()
        else {
            return false
        }
        return true
    }

    func checkCredentialImport() {
        Task {
            do {
                let isImported: Bool
#if os(iOS)
                guard let networkEnv = configurationManager.networkEnv else { return }
                isImported = try await NymVpnAccountStorage(
                    dataDir: PathManager.dataFolderURL().path(),
                    environment: networkEnv
                ).isAccountMnemonicStored()
#elseif os(macOS)
                isImported = try await grpcManager.isAccountStored()
#endif
                updateIsCredentialImported(with: isImported)
            } catch {
                logger.error("Failed to check credential import: \(error.localizedDescription)")
                updateIsCredentialImported(with: false)
            }
            await updateDeviceIdentifier()
            await updateAccountIdentifier()
            await updateAccountSummary()
        }
    }

    func updateIsCredentialImported(with value: Bool) {
        Task { @MainActor in
            guard appSettings.isCredentialImported != value else { return }
            appSettings.isCredentialImported = value
        }
    }
}

private extension CredentialsManager {
    func updateDeviceIdentifier() async {
#if os(iOS)
        guard let networkEnv = configurationManager.networkEnv else { return }
        deviceIdentifier = try? await NymVpnAccountStorage(
            dataDir: PathManager.dataFolderURL().path(),
            environment: networkEnv
        ).getDeviceIdentity()
#elseif os(macOS)
        deviceIdentifier = try? await grpcManager.deviceIdentifier()
#endif
    }

    func updateAccountIdentifier() async {
        let newAccIdentifier: String?
#if os(iOS)
        newAccIdentifier = try? await NymVpnAccountStorage(
            dataDir: PathManager.dataFolderURL().path(),
            environment: configurationManager.networkEnv ?? .newWithMainnetFallback()
        ).getAccountIdentity()
#elseif os(macOS)
        newAccIdentifier = try? await grpcManager.accountIdentifier()
#endif
        Task { @MainActor in
            accountIdentifier = newAccIdentifier
        }
    }
}
