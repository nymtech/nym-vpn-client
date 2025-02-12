import GRPC
import SwiftProtobuf
import Constants
import ConnectionTypes

extension GRPCManager {
    public func connect(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        isTwoHopEnabled: Bool,
        isZknymEnabled: Bool?
    ) async throws {
        logger.log(level: .info, "Connecting")

        return try await withCheckedThrowingContinuation { continuation in
            var request = Nym_Vpn_ConnectRequest()
            request.userAgent = userAgent

            request.entry = entryNode(from: entryGateway)
            request.exit = exitNode(from: exitRouter)

            request.disableRouting = false
            request.enableTwoHop = isTwoHopEnabled
            request.disableBackgroundCoverTraffic = false
            request.enableCredentialsMode = isZknymEnabled ?? false

            let call = client.vpnConnect(request, callOptions: nil)

            call.response.whenComplete { [weak self] result in
                switch result {
                case .success(let response):
                    if response.hasError {
                        if response.error.kind == .noAccountStored {
                            self?.generalError = GeneralNymError.noMnemonicStored
                            continuation.resume(throwing: GeneralNymError.noMnemonicStored)
                        } else if !response.error.zkNymError.isEmpty,
                                  let firstError = response.error.zkNymError.first,
                                  firstError.hasMessage {
                            continuation.resume(throwing: GeneralNymError.library(message: firstError.message))
                        } else {
                            continuation.resume(throwing: GeneralNymError.library(message: response.error.message))
                        }
                    } else {
                        continuation.resume()
                    }
                case .failure(let error):
                    self?.logger.log(level: .info, "Failed to connect to VPN: \(error)")
                }
            }
        }
    }
}

private extension GRPCManager {
    // TODO: add lowLatencyCountry support
    func entryNode(from entryGateway: EntryGateway) -> Nym_Vpn_EntryNode {
        var entryNode = Nym_Vpn_EntryNode()
        switch entryGateway {
        case let .country(country):
            var location = Nym_Vpn_Location()
            location.twoLetterIsoCountryCode = country.code
            entryNode.location = location
        case let .lowLatencyCountry(country):
            print("Add .lowLatencyCountry support")
            var location = Nym_Vpn_Location()
            location.twoLetterIsoCountryCode = country.code
            entryNode.location = location
        case let .gateway(identifier):
            var gateway = Nym_Vpn_Gateway()
            gateway.id = identifier
            entryNode.gateway = gateway
        case .randomLowLatency:
            entryNode.randomLowLatency = Google_Protobuf_Empty()
        case .random:
            entryNode.random = Google_Protobuf_Empty()
        }
        return entryNode
    }

    func exitNode(from exitRouter: ExitRouter) -> Nym_Vpn_ExitNode {
        var exitNode = Nym_Vpn_ExitNode()
        switch exitRouter {
        case let .country(country):
            var location = Nym_Vpn_Location()
            location.twoLetterIsoCountryCode = country.code
            exitNode.location = location
        case let .gateway(identifier):
            var gateway = Nym_Vpn_Gateway()
            gateway.id = identifier
            exitNode.gateway = gateway
        }
        return exitNode
    }
}
