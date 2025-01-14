import Combine
import Logging
import Foundation
import AppSettings
import Constants
#if os(iOS)
import ErrorHandler
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
import HelperInstallManager
#endif

public final class CredentialsManager {
    private let logger = Logger(label: "CredentialsManager")
#if os(macOS)
    private let grpcManager = GRPCManager.shared
    private let helperInstallManager = HelperInstallManager.shared
#endif
    private let appSettings = AppSettings.shared

    private var cancellables = Set<AnyCancellable>()

    public static let shared = CredentialsManager()

    public var deviceIdentifier: String?

    public var isValidCredentialImported: Bool {
        appSettings.isCredentialImported
    }

    private init() {
        setup()
    }

    public func add(credential: String) async throws {
        try await Task(priority: .background) {
            do {
#if os(iOS)
                let dataFolderURL = try dataFolderURL()

                if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
                    try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
                }

                try storeAccountMnemonicRaw(mnemonic: credential, path: dataFolderURL.path())
#elseif os(macOS)
                try? await helperInstallManager.installIfNeeded()
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

    public func removeCredential() async throws {
        do {
#if os(iOS)
            let dataFolderURL = try dataFolderURL()
            try forgetAccountRaw(path: dataFolderURL.path())
#endif

#if os(macOS)
            try? await helperInstallManager.installIfNeeded()
            try await grpcManager.forgetAccount()
#endif
            checkCredentialImport()
        } catch {
            // TODO: need modal for alerts
            throw error
        }
    }

    public func dataFolderURL() throws -> URL {
        guard let dataFolderURL = FileManager.default
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("Data")
        else {
            throw CredentialsManagerError.cannotCreateDB
        }
        return dataFolderURL
    }
}

private extension CredentialsManager {
    func setup() {
        setupGRPCManagerObservers()
        checkCredentialImport()
    }

    func setupGRPCManagerObservers() {
#if os(macOS)
        grpcManager.$lastError.sink { [weak self] error in
            guard let self,
                  error == GeneralNymError.noMnemonicStored
            else {
                return
            }
            Task { @MainActor in
                self.appSettings.isCredentialImported = false
            }
        }
        .store(in: &cancellables)

        helperInstallManager.$daemonState.sink { [weak self] state in
            guard state == .running else { return }
            self?.checkCredentialImport()
        }
        .store(in: &cancellables)
#endif
    }
}

private extension CredentialsManager {
    func checkCredentialImport() {
        Task(priority: .background) {
            do {
                let isImported: Bool
#if os(iOS)
                let dataFolderURL = try dataFolderURL()
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
        }
    }

    func updateIsCredentialImported(with value: Bool) {
        Task { @MainActor in
            appSettings.isCredentialImported = value
        }
    }
}

private extension CredentialsManager {
    func updateDeviceIdentifier() {
        Task(priority: .background) {
#if os(iOS)
            let dataFolderURL = try dataFolderURL()
            deviceIdentifier = try? getDeviceIdentityRaw(path: dataFolderURL.path())
#elseif os(macOS)
            deviceIdentifier = try? await grpcManager.deviceIdentifier()
#endif
        }
    }
}
