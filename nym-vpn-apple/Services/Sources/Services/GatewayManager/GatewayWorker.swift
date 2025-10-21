import AppSettings
import ConfigurationManager
import ConnectionTypes
import CountriesManagerTypes
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif

actor GatewayWorker {
    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
#if os(macOS)
    let grpcManager: GRPCManager
#endif

#if os(iOS)
    init(appSettings: AppSettings, configurationManager: ConfigurationManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }
#elseif os(macOS)
    init(appSettings: AppSettings, configurationManager: ConfigurationManager, grpcManager: GRPCManager) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
    }
#endif

#if os(iOS)
    func fetchGateways() throws -> (entry: [GatewayNode], exit: [GatewayNode], vpn: [GatewayNode]) {
        let entryNodes = try getGateways(gwType: .mixnetEntry)
        let exitNodes = try getGateways(gwType: .mixnetExit)
        let vpnNodes = try getGateways(gwType: .wg)

        let entry = entryNodes.map { GatewayNode(with: $0) }
        let exit = exitNodes.map { GatewayNode(with: $0) }
        let vpn = vpnNodes.map { GatewayNode(with: $0) }

        return (entry, exit, vpn)
    }
#elseif os(macOS)
    func fetchGateways() async throws -> (entry: [GatewayNode], exit: [GatewayNode], vpn: [GatewayNode]) {
        let entry = try await grpcManager.gateways(for: .entry)
        let exit = try await grpcManager.gateways(for: .exit)
        let vpn = try await grpcManager.gateways(for: .vpn)
        return (entry, exit, vpn)
    }
#endif

    func countries(from nodes: [GatewayNode]) -> [Country] {
        var regionsByCode: [String: Set<String>] = [:]
        nodes.compactMap(\.location).forEach { location in
            let code = location.twoLetterIsoCountryCode.uppercased()
            let region = location.region.trimmingCharacters(in: .whitespacesAndNewlines)
            guard !region.isEmpty else { return }
            regionsByCode[code, default: []].insert(region)
        }
        var result: [Country] = []
        result.reserveCapacity(regionsByCode.count)

        for (code, regionsSet) in regionsByCode {
            let regions = regionsSet.sorted {
                $0.localizedCaseInsensitiveCompare($1) == .orderedAscending
            }
            result.append(Country(name: code, code: code, regions: regions)) // name will be localized on main
        }

        result.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        return result
    }
}
