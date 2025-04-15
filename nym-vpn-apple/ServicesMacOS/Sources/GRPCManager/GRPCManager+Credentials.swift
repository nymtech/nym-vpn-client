import Constants
import GRPC
import SwiftProtobuf
import ErrorReason

extension GRPCManager {
    public func storeAccount(with mnemonic: String) async throws {
        logger.log(level: .info, "Importing credentials")

        return try await withCheckedThrowingContinuation { continuation in
            let call = client.storeAccount(storeAccountRequest(with: mnemonic))

            call.response.whenComplete { result in
                switch result {
                case .success(let response):
                    if response.hasError, let errorDetail = response.error.errorDetail {
                        switch errorDetail {
                        case let .errorResponse(apiErrorResponse):
                            continuation.resume(throwing: ErrorReason.api(apiErrorResponse.message))
                        case let .internal(message),
                            let .invalidMnemonic(message),
                            let .storageError(message),
                            let .unexpectedResponse(message):
                            continuation.resume(throwing: GeneralNymError.library(message: message))
                        }
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
                    if response.hasError, let errorDetail = response.error.errorDetail {
                        switch errorDetail {
                        case let .errorResponse(apiErrorResponse):
                            continuation.resume(throwing: ErrorReason.api(apiErrorResponse.message))
                        case .registrationInProgress(_):
                            continuation.resume(throwing: ErrorReason.registrationInProgress)
                        case let .unexpectedResponse(message),
                            let .removeAccount(message),
                            let .removeDeviceKeys(message),
                            let .resetCredentialStore(message),
                            let .removeAccountFiles(message),
                            let .initDeviceKeys(message),
                            let .internal(message):
                            continuation.resume(throwing: GeneralNymError.library(message: message))
                        }
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
                case let .success(response):
                    continuation.resume(
                        returning: (
                            account: response.account.url,
                            signIn: response.signIn.url,
                            signUp: response.signUp.url
                        )
                    )
                case let .failure(error):
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
