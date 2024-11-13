import Constants
import GRPC

extension GRPCManager {
    public func storeAccount(with mnemonic: String) async throws {
        guard helperManager.isHelperAuthorizedAndRunning()
        else {
            throw GRPCError.daemonNotRunning
        }

        logger.log(level: .info, "Importing credentials")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.storeAccount(storeAccountRequest(with: mnemonic))

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    if response.hasError {
                        
                        continuation.resume(throwing: GeneralNymError.library(message: response.error.message))
                        break
                    }
                    continuation.resume()
                case .failure(let error):
                    continuation.resume(throwing: error)
                }
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
