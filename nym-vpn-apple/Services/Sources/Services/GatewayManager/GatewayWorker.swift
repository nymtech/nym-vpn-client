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

    func countries(from nodes: [GatewayNode]) -> [NymCountry] {
        // countryCode → regionName → Set<cityName>
        var citiesByRegionByCountry: [String: [String: Set<String>]] = [:]
        nodes.compactMap(\.location).forEach { location in
            let code = location.twoLetterIsoCountryCode.uppercased()
            let region = location.region.trimmingCharacters(in: .whitespacesAndNewlines)
            let city = location.city.trimmingCharacters(in: .whitespacesAndNewlines)

            guard !region.isEmpty, !city.isEmpty else { return }

            citiesByRegionByCountry[code, default: [:]][region, default: []].insert(city)
        }

        var result: [NymCountry] = []
        result.reserveCapacity(citiesByRegionByCountry.count)

        for (code, regionsDict) in citiesByRegionByCountry {
            let regions: [NymCountry.Region] = regionsDict
                .map { regionName, citiesSet in
                    let cities = citiesSet.sorted {
                        $0.localizedCaseInsensitiveCompare($1) == .orderedAscending
                    }
                    return NymCountry.Region(name: regionName, cities: cities)
                }
                .sorted { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }

            result.append(NymCountry(name: code, code: code, regions: regions)) // name localized later
        }
        result.sort { $0.name.localizedCaseInsensitiveCompare($1.name) == .orderedAscending }
        return result
    }
}
