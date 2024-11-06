import Constants
import GRPC

extension GRPCManager {
    public func storeAccount(with mnemonic: String) throws {
        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            throw GRPCError.daemonNotRunning
        }

        logger.log(level: .info, "Importing credentials")

        let request = storeAccountRequest(with: mnemonic)
        let call = client.storeAccount(request)

        var isCredentialImported = false
        var errorMessage: String?

        call.response.whenComplete { result in
            switch result {
            case let .success(response):
                isCredentialImported = response.success
                errorMessage = response.error.message
            case let .failure(error):
                isCredentialImported = false
                errorMessage = error.localizedDescription
            }
        }

        do {
            _ = try call.status.wait()
            if !isCredentialImported {
                logger.log(level: .error, "Failed to store account with: \(String(describing: errorMessage))")
                throw GeneralNymError.library(message: errorMessage ?? "")
            }
        }
    }

    public func removeAccount() async throws -> Bool {
        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            throw GRPCError.daemonNotRunning
        }

        logger.log(level: .info, "Removing credentials")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.removeAccount(Nym_Vpn_RemoveAccountRequest())

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    if response.hasError {
                        continuation.resume(throwing: GeneralNymError.library(message: response.error.message))
                        break
                    }
                    continuation.resume(returning: response.success)
                case .failure(let error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }

    public func isAccountStored() async throws -> Bool {
        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            throw GRPCError.daemonNotRunning
        }
        logger.log(level: .info, "Checking if stored account")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.isAccountStored(
                Nym_Vpn_IsAccountStoredRequest(),
                callOptions: CallOptions(timeLimit: .timeout(.seconds(5)))
            )

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    continuation.resume(returning: response.isStored)
                case .failure(let error):
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}

private extension GRPCManager {
    func storeAccountRequest(with mnemonic: String) -> Nym_Vpn_StoreAccountRequest {
        var request = Nym_Vpn_StoreAccountRequest()
        request.mnemonic = mnemonic
        request.nonce = 0
        return request
    }
}
