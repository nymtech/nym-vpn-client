import Constants

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
            case .success(let response):
                isCredentialImported = response.success
                errorMessage = response.error.message
            case .failure(let error):
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
}

private extension GRPCManager {
    func storeAccountRequest(with mnemonic: String) -> Nym_Vpn_StoreAccountRequest {
        var request = Nym_Vpn_StoreAccountRequest()
        request.mnemonic = mnemonic
        request.nonce = 0
        return request
    }
}
