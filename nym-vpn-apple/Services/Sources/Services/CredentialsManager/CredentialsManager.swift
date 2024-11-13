import Combine
import Logging
import Foundation
import AppSettings
import Constants
import ErrorHandler

#if os(iOS)
import MixnetLibrary
#endif

#if os(macOS)
import GRPCManager
import HelperManager
#endif

public final class CredentialsManager {
    private let logger = Logger(label: "CredentialsManager")
#if os(macOS)
    private let grpcManager = GRPCManager.shared
    private let helperManager = HelperManager.shared
#endif
    private let appSettings = AppSettings.shared

    private var cancellables = Set<AnyCancellable>()

    public static let shared = CredentialsManager()

    public var isValidCredentialImported: Bool {
        appSettings.isCredentialImported
    }

    private init() {
        setup()
    }

    public func add(credential: String) async throws {
        let trimmedCredential = credential.trimmingCharacters(in: .whitespacesAndNewlines)
        try await Task(priority: .background) {
            do {
#if os(iOS)
                let dataFolderURL = try dataFolderURL()

                if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
                    try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
                }
                try storeAccountMnemonic(mnemonic: trimmedCredential, path: dataFolderURL.path())
#elseif os(macOS)
                // TODO: check if daemon is installed and does not need an update
                _ = await installHelperIfNeeded()
                try await grpcManager.storeAccount(with: trimmedCredential)
#endif
                checkCredentialImport()
            } catch {
                if let vpnError = error as? VpnError {
                    throw VPNErrorReason(with: vpnError)
                } else {
                    throw error
                }
            }
        }.value
    }

    public func removeCredential() async throws {
        do {
            let removalResult: Bool
#if os(iOS)
            let dataFolderURL = try dataFolderURL()
            removalResult = try removeAccountMnemonic(path: dataFolderURL.path())
            // TODO: remove tunnel as well
#endif

#if os(macOS)
            _ = await installHelperIfNeeded()
            removalResult = try await grpcManager.removeAccount()
#endif
            checkCredentialImport()
        } catch {
            // TODO: need modal for alerts
            print(" remove credential : \(error)")
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
                isImported = try isAccountMnemonicStored(path: dataFolderURL.path())
#endif

#if os(macOS)
                isImported = try await grpcManager.isAccountStored()
#endif
                updateIsCredentialImported(with: isImported)
            } catch {
                print("checkCredentialImport error: \(error)")
                updateIsCredentialImported(with: false)
            }
        }
    }

    func updateIsCredentialImported(with value: Bool) {
        Task { @MainActor in
            appSettings.isCredentialImported = value
        }
    }
}

#if os(macOS)
private extension CredentialsManager {
    func installHelperIfNeeded() async -> Bool {
        var isInstalledAndRunning = helperManager.isHelperAuthorizedAndRunning()
        // TODO: check if possible to split is helper running vs isHelperAuthorized
        guard isInstalledAndRunning && !grpcManager.requiresUpdate
        else {
            do {
                isInstalledAndRunning = try await helperManager.installHelperIfNeeded()
            } catch let error {
                logger.error("Failed to install helper: \(error)")
            }
            return isInstalledAndRunning
        }
        return isInstalledAndRunning
    }
}
#endif
