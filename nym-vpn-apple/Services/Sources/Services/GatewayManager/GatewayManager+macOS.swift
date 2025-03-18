#if os(macOS)
import Foundation
import CountriesManagerTypes

extension GatewayManager {
    func fetchGateways() async {
        do {
            let entryGateways = try await grpcManager.gateways(for: .entry)
            let exitGateways = try await grpcManager.gateways(for: .exit)
            let vpnGateways = try await grpcManager.gateways(for: .vpn)

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
        } catch {
            logger.error("Failed to fetch: \(error.localizedDescription)")
        }
        logger.info("Loaded gateways:")
        logger.info("entry: \(gatewayStore.entry.count)")
        logger.info("exit: \(gatewayStore.exit.count)")
        logger.info("vpn: \(gatewayStore.vpn.count)")
    }
}
#endif
