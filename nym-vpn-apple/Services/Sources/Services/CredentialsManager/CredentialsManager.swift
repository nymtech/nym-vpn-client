import Combine
import Foundation
import AppSettings
import Constants

#if os(iOS)
import MixnetLibrary
#endif

#if os(macOS)
import GRPCManager
import HelperManager
#endif

public final class CredentialsManager {
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

    deinit {
#if os(iOS)
        do {
            try stopAccountController()
        } catch {
            print("Error stopping account controller: \(error)")
        }
#endif
    }

    public func add(credential: String) async throws {
        Task {
            let trimmedCredential = credential.trimmingCharacters(in: .whitespacesAndNewlines)
            do {
#if os(iOS)
                let dataFolderURL = try dataFolderURL()

                if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
                    try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
                }
                try storeAccountMnemonic(mnemonic: trimmedCredential, path: dataFolderURL.path())
#endif

#if os(macOS)
                _ = try await helperManager.installHelperIfNeeded()
                try grpcManager.storeAccount(with: trimmedCredential)
#endif
                checkCredentialImport()
            } catch let error {
                print("add credential : \(error)")
                throw error
            }
        }
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
            _ = try await helperManager.installHelperIfNeeded()
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
        setupAccountController()
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

    func setupAccountController() {
        Task {
#if os(iOS)
            do {
                let dataFolderURL = try dataFolderURL()
                try startAccountController(dataDir: dataFolderURL.path())
            } catch {
                print("Error starting account controller: \(error)")
            }
#endif
        }
    }
}

private extension CredentialsManager {
    func checkCredentialImport() {
        Task {
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
