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
            try importCredential(credential: trimmedCredential, path: dataFolderURL.path())
#endif
#if os(macOS)
            try grpcManager.importCredential(credential: trimmedCredential)
#endif
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
            guard error == GeneralNymError.invalidCredential else { return }
            self?.appSettings.isCredentialImported = false
        }
        .store(in: &cancellables)
#endif
    }
}
