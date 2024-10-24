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

    public func add(credential: String) async throws {
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
            Task { @MainActor in
                appSettings.isCredentialImported = true
            }
        } catch let error {
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

extension CredentialsManager {
    func setup() {
        setupGRPCManagerObservers()
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
