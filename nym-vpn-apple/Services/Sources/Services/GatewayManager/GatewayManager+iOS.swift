#if os(iOS)
import Foundation
import AppVersionProvider
import NymVPNLib
import CountriesManagerTypes

extension GatewayManager {
    @MainActor func fetchGateways() async {
        Task {
            do {
                let entryNodes = try getGateways(gwType: .mixnetEntry)
                let exitNodes = try getGateways(gwType: .mixnetExit)
                let vpnNodes = try getGateways(gwType: .wg)

                let entryGateways = entryNodes.map { GatewayNode(with: $0) }
                let exitGateways = exitNodes.map { GatewayNode(with: $0) }
                let vpnGateways = vpnNodes.map { GatewayNode(with: $0) }

                guard !entryGateways.isEmpty, !exitGateways.isEmpty, !vpnGateways.isEmpty
                else {
                    logger.info("Empty gateways from API")
                    return
                }
                entry = entryGateways
                exit = exitGateways
                vpn = vpnGateways

                gatewayStore.entry = entryGateways
                gatewayStore.exit = exitGateways
                gatewayStore.vpn = vpnGateways

                gatewayStore.lastFetchDate = Date()

                storeGatewayStore()
                updateCountriesFromGateways()
                isLoading = false
            } catch {
                logger.error("Failed to fetch: \(error.localizedDescription)")
            }
            logger.info("Loaded gateways:")
            logger.info("entry: \(gatewayStore.entry.count)")
            logger.info("exit: \(gatewayStore.exit.count)")
            logger.info("vpn: \(gatewayStore.vpn.count)")
        }
    }
}
#endif
