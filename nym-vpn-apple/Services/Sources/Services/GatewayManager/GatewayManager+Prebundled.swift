import Foundation
import AppSettings
import Constants
import CountriesManagerTypes

extension GatewayManager {
    func loadGatewayStore() {
        guard let gatewayStoreString = appSettings.gatewayStore,
              let loadedGatewayStore = GatewayNodeStore(rawValue: gatewayStoreString)
        else {
            return
        }
        gatewayStore = loadedGatewayStore
        entry = loadedGatewayStore.entry
        exit = loadedGatewayStore.exit
        vpn = loadedGatewayStore.vpn
    }

    func loadPrebundledServersIfNecessary() {
        guard entry.isEmpty || exit.isEmpty || vpn.isEmpty else { return }
        guard let entryServersURL = Bundle.main.url(forResource: "gatewaysEntry", withExtension: "json"),
              let exitServersURL = Bundle.main.url(forResource: "gatewaysExit", withExtension: "json"),
              let vpnServersURL = Bundle.main.url(forResource: "gatewaysVpn", withExtension: "json")
        else {
            updateError(with: GeneralNymError.noPrebundledCountries)
            return
        }

        do {
            let prebundledEntryServers = try loadPrebundledServers(from: entryServersURL)
            let prebundledExitServers = try loadPrebundledServers(from: exitServersURL)
            let prebundledVPNServers = try loadPrebundledServers(from: vpnServersURL)

            gatewayStore.entry = prebundledEntryServers
            gatewayStore.exit = prebundledExitServers
            gatewayStore.vpn = prebundledVPNServers

            entry = prebundledEntryServers
            exit = prebundledExitServers
            vpn = prebundledVPNServers

            logger.info("Loading prebundled servers")
            logger.info("entry: \(gatewayStore.entry.count)")
            logger.info("exit: \(gatewayStore.exit.count)")
            logger.info("vpn: \(gatewayStore.vpn.count)")
        } catch let error {
            updateError(with: error)
            return
        }
    }

    func loadPrebundledServers(from fileURL: URL) throws -> [GatewayNode] {
        do {
            let data = try Data(contentsOf: fileURL)
            let nodes = try JSONDecoder().decode([Node].self, from: data)
            let servers = nodes.map {
                // TODO: update scores with actual thresholds
                GatewayNode(
                    id: $0.identityKey,
                    countryCode: $0.location.twoLetterISOCountryCode,
                    wgScore: .high,
                    mixnetScore: .high,
                    moniker: $0.name
                )
            }
            return servers
        } catch {
            throw GeneralNymError.cannotParseCountries
        }
    }

    func storeGatewayStore() {
        Task { @MainActor in
            appSettings.gatewayStore = gatewayStore.rawValue
        }
    }
}

struct Node: Codable {
    let identityKey: String
    let name: String
    let authenticator: Authenticator
    let ipPacketRouter: IPPacketRouter
    let location: Location
    let lastProbe: LastProbe
    let ipAddresses: [String]
    let mixPort: Int
    let role: String
    let entry: Entry
    let performance: String
    let buildInformation: BuildInformation

    enum CodingKeys: String, CodingKey {
        case identityKey = "identity_key"
        case name
        case authenticator
        case ipPacketRouter = "ip_packet_router"
        case location
        case lastProbe = "last_probe"
        case ipAddresses = "ip_addresses"
        case mixPort = "mix_port"
        case role
        case entry
        case performance
        case buildInformation = "build_information"
    }
}

struct Authenticator: Codable {
    let address: String
}

struct IPPacketRouter: Codable {
    let address: String
}

struct Location: Codable {
    let twoLetterISOCountryCode: String
    let latitude: Double
    let longitude: Double

    enum CodingKeys: String, CodingKey {
        case twoLetterISOCountryCode = "two_letter_iso_country_code"
        case latitude, longitude
    }
}

struct LastProbe: Codable {
    let lastUpdatedUTC: String
    let outcome: Outcome

    enum CodingKeys: String, CodingKey {
        case lastUpdatedUTC = "last_updated_utc"
        case outcome
    }
}

struct Outcome: Codable {
    let asEntry: [String: Bool]
    let asExit: [String: Bool]
    let wg: Wg

    enum CodingKeys: String, CodingKey {
        case asEntry = "as_entry"
        case asExit = "as_exit"
        case wg
    }
}

struct Wg: Codable {
    let canHandshakeV4: Bool
    let canHandshakeV6: Bool
    let canRegister: Bool
    let canResolveDNSV4: Bool
    let canResolveDNSV6: Bool
    let pingHostsPerformanceV4: Double
    let pingHostsPerformanceV6: Double
    let pingIpsPerformanceV4: Double
    let pingIpsPerformanceV6: Double
    let canHandshake: Bool
    let canResolveDNS: Bool
    let pingHostsPerformance: Double
    let pingIpsPerformance: Double

    enum CodingKeys: String, CodingKey {
        case canHandshakeV4 = "can_handshake_v4"
        case canHandshakeV6 = "can_handshake_v6"
        case canRegister = "can_register"
        case canResolveDNSV4 = "can_resolve_dns_v4"
        case canResolveDNSV6 = "can_resolve_dns_v6"
        case pingHostsPerformanceV4 = "ping_hosts_performance_v4"
        case pingHostsPerformanceV6 = "ping_hosts_performance_v6"
        case pingIpsPerformanceV4 = "ping_ips_performance_v4"
        case pingIpsPerformanceV6 = "ping_ips_performance_v6"
        case canHandshake = "can_handshake"
        case canResolveDNS = "can_resolve_dns"
        case pingHostsPerformance = "ping_hosts_performance"
        case pingIpsPerformance = "ping_ips_performance"
    }
}

struct Entry: Codable {
    let hostname: String?
    let wsPort: Int
    let wssPort: Int?

    enum CodingKeys: String, CodingKey {
        case hostname
        case wsPort = "ws_port"
        case wssPort = "wss_port"
    }
}

struct BuildInformation: Codable {
    let buildVersion: String
    let commitBranch: String
    let commitSha: String

    enum CodingKeys: String, CodingKey {
        case buildVersion = "build_version"
        case commitBranch = "commit_branch"
        case commitSha = "commit_sha"
    }
}
