import Combine
import Foundation
import AppSettings
import Constants

#if os(iOS)
import MixnetLibrary
#endif

#if os(macOS)
import GRPCManager
#endif

public final class CredentialsManager {
#if os(macOS)
    private let grpcManager = GRPCManager.shared
#endif
    private let appSettings = AppSettings.shared

    private var cancellables = Set<AnyCancellable>()

    public static let shared = CredentialsManager()

    public var isValidCredentialImported: Bool {
        guard let expiryDate = appSettings.credentialExpiryDate, appSettings.isCredentialImported else { return false }
        let isValid = Date() < expiryDate
        return appSettings.isCredentialImported && isValid
    }

    public var expiryDate: Date? {
        appSettings.credentialExpiryDate
    }

    public var startDate: Date? {
        appSettings.credentialStartDate
    }

    private init() {
        setup()
    }

    public func add(credential: String) throws {
        let trimmedCredential = credential.trimmingCharacters(in: .whitespacesAndNewlines)
        do {
#if os(iOS)
            let dataFolderURL = try dataFolderURL()

            if !FileManager.default.fileExists(atPath: dataFolderURL.path()) {
                try FileManager.default.createDirectory(at: dataFolderURL, withIntermediateDirectories: true)
            }
            let expiryDate = try importCredential(credential: trimmedCredential, path: dataFolderURL.path())
            guard let expiryDate
            else {
                throw CredentialsManagerError.noExpiryDate
            }
#endif
#if os(macOS)
            let expiryDate = try grpcManager.importCredential(credential: trimmedCredential)
#endif
            appSettings.isCredentialImported = true
            appSettings.credentialExpiryDate = expiryDate
            appSettings.credentialStartDate = Date()
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
                  error == GeneralNymError.invalidCredential
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
