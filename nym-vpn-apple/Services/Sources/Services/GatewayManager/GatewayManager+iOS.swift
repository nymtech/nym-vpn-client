#if os(iOS)
import Foundation
import AppVersionProvider
import NymVPNLib
import CountriesManagerTypes

extension GatewayManager {
    @MainActor func fetchGateways() async {
        Task { [weak self] in
            guard let self else { return }
            try? await Task.sleep(for: .seconds(5))
            do {
                let entryNodes = try getGateways(gwType: .mixnetEntry)
                let exitNodes  = try getGateways(gwType: .mixnetExit)
                let vpnNodes   = try getGateways(gwType: .wg)

                let entryGateways = entryNodes.map { GatewayNode(with: $0) }
                let exitGateways = exitNodes.map { GatewayNode(with: $0) }
                let vpnGateways = vpnNodes.map { GatewayNode(with: $0) }

                guard !entryGateways.isEmpty, !exitGateways.isEmpty, !vpnGateways.isEmpty else {
                    self.logger.info("Empty gateways from API")
                    return
                }

                await MainActor.run {
                    self.entry = entryGateways
                    self.exit = exitGateways
                    self.vpn = vpnGateways

                    self.gatewayStore.entry = entryGateways
                    self.gatewayStore.exit = exitGateways
                    self.gatewayStore.vpn = vpnGateways

                    self.gatewayStore.lastFetchDate = Date()

                    self.storeGatewayStore()
                    self.updateCountriesFromGateways()
                    self.isLoading = false
                }

                self.logger.info("Loaded gateways:")
                self.logger.info("entry: \(self.gatewayStore.entry.count)")
                self.logger.info("exit: \(self.gatewayStore.exit.count)")
                self.logger.info("vpn: \(self.gatewayStore.vpn.count)")
            } catch {
                self.logger.error("Failed to fetch: \(error.localizedDescription)")
            }
        }
    }
}
#endif
