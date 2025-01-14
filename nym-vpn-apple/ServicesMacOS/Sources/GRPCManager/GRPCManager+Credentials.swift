import Constants
import GRPC
import SwiftProtobuf

extension GRPCManager {
    public func storeAccount(with mnemonic: String) async throws {
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

    public func forgetAccount() async throws {
        logger.log(level: .info, "Forgetting credentials")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.forgetAccount(Google_Protobuf_Empty())

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    if response.hasError {
                        continuation.resume(throwing: GeneralNymError.library(message: response.error.message))
                    } else {
                        continuation.resume()
                    }
                case .failure(let error):
                    continuation.resume(
                        throwing:
                            GeneralNymError.library(message: error.localizedDescription)
                    )
                }
            }
        }
    }

    public func isAccountStored() async throws -> Bool {
        logger.log(level: .info, "Checking if stored account")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.isAccountStored(
                Google_Protobuf_Empty(),
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

    public func accountLinks() async throws -> (account: String, signIn: String, signUp: String) {
        logger.log(level: .info, "Fetching account links")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.getAccountLinks(Nym_Vpn_GetAccountLinksRequest())

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    continuation.resume(
                        returning: (
                            account: response.links.account.url,
                            signIn: response.links.signIn.url,
                            signUp: response.links.signUp.url
                        )
                    )
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
