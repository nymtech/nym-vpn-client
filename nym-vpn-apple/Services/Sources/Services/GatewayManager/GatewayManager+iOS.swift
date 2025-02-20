#if os(iOS)
import Foundation
import AppVersionProvider
import MixnetLibrary
import CountriesManagerTypes

extension GatewayManager {
    func fetchGateways() async {
        let userAgent = UserAgent(
            application: AppVersionProvider.app,
            version: "\(AppVersionProvider.appVersion()) (\(AppVersionProvider.libVersion))",
            platform: AppVersionProvider.platform,
            gitCommit: ""
        )
        Task(priority: .background) {
            do {
                let entryNodes = try getGateways(gwType: .mixnetEntry, userAgent: userAgent, minGatewayPerformance: nil)
                let exitNodes = try getGateways(gwType: .mixnetExit, userAgent: userAgent, minGatewayPerformance: nil)
                let vpnNodes = try getGateways(gwType: .wg, userAgent: userAgent, minGatewayPerformance: nil)

                let entryGateways = entryNodes.map { GatewayNode(with: $0) }
                let exitGateways = exitNodes.map { GatewayNode(with: $0) }
                let vpnGateways = vpnNodes.map { GatewayNode(with: $0) }

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
}
#endif
