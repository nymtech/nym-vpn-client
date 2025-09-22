import GRPC
import SwiftProtobuf
import Constants
import ConnectionTypes

extension GRPCManager {
    public func connect(
        entryGateway: EntryGateway,
        exitRouter: ExitRouter,
        isTwoHopEnabled: Bool,
        disableIPv6: Bool
    ) async throws {
        var request = NymVpnService_ConnectRequest()
        request.userAgent = userAgent

        request.entry = entryNode(from: entryGateway)
        request.exit = exitNode(from: exitRouter)

        request.enableTwoHop = isTwoHopEnabled
        request.disableBackgroundCoverTraffic = false
        request.disableIpv6 = disableIPv6

        _ = try await client.connectTunnel(request)
    }

    public func disconnect() async throws {
        _ = try await client.disconnectTunnel(Google_Protobuf_Empty())
    }
}

private extension GRPCManager {
    // TODO: add lowLatencyCountry support
    func entryNode(from entryGateway: EntryGateway) -> NymVpnService_EntryNode {
        var entryNode = NymVpnService_EntryNode()
        switch entryGateway {
        case let .country(country):
            var location = NymVpnService_Country()
            location.twoLetterIsoCountryCode = country.code
            entryNode.country = location
        case let .lowLatencyCountry(country):
            print("Add .lowLatencyCountry support")
            var location = NymVpnService_Country()
            location.twoLetterIsoCountryCode = country.code
            entryNode.country = location
        case let .gateway(node):
            var gateway = NymVpnService_GatewayId()
            gateway.id = node.id
            entryNode.gateway = gateway
        case .random:
            entryNode.random = Google_Protobuf_Empty()
        }
        return entryNode
    }

    func exitNode(from exitRouter: ExitRouter) -> NymVpnService_ExitNode {
        var exitNode = NymVpnService_ExitNode()
        switch exitRouter {
        case let .country(country):
            var location = NymVpnService_Country()
            location.twoLetterIsoCountryCode = country.code
            exitNode.country = location
        case let .gateway(node):
            var gateway = NymVpnService_GatewayId()
            gateway.id = node.id
            exitNode.gateway = gateway
        }
        return exitNode
    }
}
