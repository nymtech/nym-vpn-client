import Combine
import Logging
import Foundation
import AppSettings
import Constants
import ErrorReason
#if os(iOS)
import ErrorHandler
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif

@MainActor public final class CredentialsManager: ObservableObject {
    private let logger = Logger(label: "CredentialsManager")
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    private let appSettings = AppSettings.shared

    private var cancellables = Set<AnyCancellable>()

    public static let shared = CredentialsManager()

    public var deviceIdentifier: String?
    @Published public var accountIdentifier: String?

    public var isValidCredentialImported: Bool {
        appSettings.isCredentialImported
    }

    public var accountToken: String? {
        appSettings.accountToken
    }

    private init() {
        setup()
    }

    public func add(credential: String) async throws {
        try await Task {
            do {
#if os(iOS)
                let dataFolderURL = try Self.dataFolderURL()
                try loginRaw(mnemonic: credential, path: dataFolderURL.path())
#elseif os(macOS)
                try await grpcManager.storeAccount(with: credential)
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
            let dataFolderURL = try Self.dataFolderURL()
            try createAccountRaw(path: dataFolderURL.path())
            Task { @MainActor in
                checkCredentialImport()
            }
        }.value
#endif
    }

    public func mnemonic() async throws -> String {
#if os(iOS)
        try await Task {
            let dataFolderURL = try Self.dataFolderURL()
            return try getStoredMnemonicRaw(path: dataFolderURL.path())
        }.value
#elseif os(macOS)
        // TODO: add missing grpc
        return ""
#endif
    }

    public func registerAccount() async throws {
#if os(iOS)
        try await Task {
            do {
                let dataFolderURL = try Self.dataFolderURL()
                let result = try registerAccountRaw(path: dataFolderURL.path())
                Task { @MainActor in
                    appSettings.accountToken = result.accountToken
                    checkCredentialImport()
                }
            }
        }.value
#endif
    }

    public func removeCredential() async throws {
        try await Task {
            do {
#if os(iOS)
                let dataFolderURL = try Self.dataFolderURL()
                try forgetAccountRaw(path: dataFolderURL.path())
#endif

#if os(macOS)
                try await grpcManager.forgetAccount()
#endif
                checkCredentialImport()
            } catch {
                // TODO: need modal for alerts
                throw error
            }
            Task { @MainActor in
                appSettings.accountToken = nil
            }
        }.value
    }

    /// Group folder, created automatically if does not exists
    /// `/private/var/mobile/Containers/Shared/AppGroup/xxx-xxx-xxx-xxx-xxx/Data/`
    /// - Returns: URL to group data folder
    public nonisolated static func dataFolderURL() throws -> URL {
        guard let dataFolderURL = FileManager.default
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("Data")
        else {
            throw CredentialsManagerError.cannotCreateDB
        }
        if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
            try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
        }
        return dataFolderURL
    }

    public nonisolated static func cacheFolderURL() throws -> URL {
        try Self.dataFolderURL().appendingPathComponent("Cache")
    }

    public nonisolated static func configFolderURL() throws -> URL {
        try Self.dataFolderURL().appendingPathComponent("Config")
    }
}

private extension CredentialsManager {
    func setup() {
#if os(iOS)
        checkCredentialImport()
#elseif os(macOS)
        setupGRPCManagerObservers()
#endif
    }

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
    func checkCredentialImport() {
        Task {
            do {
                let isImported: Bool
#if os(iOS)
                let dataFolderURL = try Self.dataFolderURL()
                isImported = try isAccountMnemonicStoredRaw(path: dataFolderURL.path())
#elseif os(macOS)
                isImported = try await grpcManager.isAccountStored()
#endif
                updateIsCredentialImported(with: isImported)
            } catch {
                logger.error("Failed to check credential import: \(error.localizedDescription)")
                updateIsCredentialImported(with: false)
            }
            updateDeviceIdentifier()
            updateAccountIdentifier()
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
    func updateDeviceIdentifier() {
        Task {
#if os(iOS)
            let dataFolderURL = try Self.dataFolderURL()
            deviceIdentifier = try? getDeviceIdentityRaw(path: dataFolderURL.path())
#elseif os(macOS)
            deviceIdentifier = try? await grpcManager.deviceIdentifier()
#endif
        }
    }

    func updateAccountIdentifier() {
        Task {
            let newAccIdentifier: String?
#if os(iOS)
            let dataFolderURL = try Self.dataFolderURL()
            newAccIdentifier = try? getAccountIdentityRaw(path: dataFolderURL.path())
#elseif os(macOS)
            newAccIdentifier = try? await grpcManager.accountIdentifier()
#endif
            Task { @MainActor in
                accountIdentifier = newAccIdentifier
            }
        }
    }
}
