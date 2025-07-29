import Constants
import GRPC
import SwiftProtobuf
import ErrorReason

extension GRPCManager {
    public func storeAccount(with mnemonic: String) async throws {
        let result = try await client.storeAccount(storeAccountRequest(with: mnemonic))
        switch result.error.errorDetail {
        case let .invalidMnemonic(message),
            let .storageError(message),
            let .unexpectedResponse(message),
            let .internal(message):
            throw GeneralNymError.library(message: message)
        case let .vpnApi(apiError):
            switch apiError.errorDetail {
            case .timeout:
                throw ErrorReason.apiTimeout
            case let .statusCode(code):
                throw ErrorReason.api(String(code))
            case let .response(errorResponse):
                throw ErrorReason.apiResponse(errorResponse.message)
            case .none:
                return
            }
        case .none:
            return
        }
    }

    public func forgetAccount() async throws {
        let result = try await client.forgetAccount(Google_Protobuf_Empty())

        switch result.error.errorDetail {
        case .registrationInProgress:
            throw ErrorReason.registrationInProgress
        case let .unexpectedResponse(message),
            let .removeAccount(message),
            let .removeDeviceKeys(message),
            let .resetCredentialStore(message),
            let .removeAccountFiles(message),
            let .initDeviceKeys(message),
            let .internal(message):
            throw GeneralNymError.library(message: message)
        case let .vpnApi(apiError):
            switch apiError.errorDetail {
            case .timeout:
                throw ErrorReason.apiTimeout
            case let .statusCode(code):
                throw ErrorReason.api(String(code))
            case let .response(errorResponse):
                throw ErrorReason.apiResponse(errorResponse.message)
            case .none:
                throw ErrorReason.unknown
            }
        case .none:
            return
        }
    }

    public func isAccountStored() async throws -> Bool {
        try await client.isAccountStored(
            Google_Protobuf_Empty(),
            callOptions: CallOptions(timeLimit: .timeout(.seconds(5)))
        ).value
    }

    public func accountLinks() async throws -> (account: String, signIn: String, signUp: String) {
        let result = try await client.getAccountLinks(NymVpnService_GetAccountLinksRequest())
        return (account: result.account.url, signIn: result.signIn.url, signUp: result.signUp.url)
    }
}

private extension GRPCManager {
    func storeAccountRequest(with mnemonic: String) -> NymVpnService_StoreAccountRequest {
        var request = NymVpnService_StoreAccountRequest()
        request.mnemonic = mnemonic
        return request
    }
}
