import Foundation
import AppSettings
import Constants
import ConnectionTypes

extension GatewayManager {
    func loadGatewayStore() {
        autoreleasepool {
            guard let gatewayStoreString = appSettings.gatewayStore,
                  let loadedGatewayStore = GatewayNodeStore(rawValue: gatewayStoreString)
            else {
                return
            }
#if SANTA
            if GatewayCacheReloadPolicy.needsReload(
                store: loadedGatewayStore,
                currentEnv: configurationManager.currentEnvString
            ) {
                clearGatewayStoreForEnvironmentChange()
                return
            }
#endif
            gatewayStore = loadedGatewayStore
            entry = loadedGatewayStore.entry
            exit = loadedGatewayStore.exit
            vpn = loadedGatewayStore.vpn

            updateCountriesFromGateways()
        }
    }

    func loadPrebundledServersIfNecessary() {
#if SANTA
        guard GatewayCacheReloadPolicy.shouldLoadPrebundledFallback(
            currentEnv: configurationManager.currentEnvString
        ) else {
            return
        }
#endif
        guard entry.isEmpty || exit.isEmpty || vpn.isEmpty else { return }
        guard let entryServersURL = Bundle.main.url(forResource: "gatewaysEntry", withExtension: "json"),
              let exitServersURL = Bundle.main.url(forResource: "gatewaysExit", withExtension: "json"),
              let vpnServersURL = Bundle.main.url(forResource: "gatewaysVpn", withExtension: "json")
        else {
            logger.error("\(GeneralNymError.noPrebundledCountries.localizedDescription)")
            return
        }

        autoreleasepool {
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

                updateCountriesFromGateways()

                logger.info("Loading prebundled servers")
                logger.info("entry: \(gatewayStore.entry.count)")
                logger.info("exit: \(gatewayStore.exit.count)")
                logger.info("vpn: \(gatewayStore.vpn.count)")
            } catch let error {
                logger.error("\(error.localizedDescription)")
                return
            }
        }
    }

    // swiftlint:disable:next function_body_length
    func loadPrebundledServers(from fileURL: URL) throws -> [GatewayNode] {
        do {
            let data = try Data(contentsOf: fileURL)

            let decoder = JSONDecoder()
            decoder.dateDecodingStrategy = .custom { decoder in
                let container = try decoder.singleValueContainer()
                let decodedString = try container.decode(String.self)

                // flexible parser with fractional seconds
                let formatter = ISO8601DateFormatter()
                formatter.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
                if let date1 = formatter.date(from: decodedString) {
                    return date1
                }

                // fallback plain ISO8601
                let formatter2 = ISO8601DateFormatter()
                if let date2 = formatter2.date(from: decodedString) {
                    return date2
                }

                throw DecodingError.dataCorruptedError(
                    in: container,
                    debugDescription: "Bad ISO8601 date: \(decodedString)"
                )
            }

            let nodes = try decoder.decode(Node.self, from: data)
            // swiftlint:disable:next closure_body_length
            return nodes.map { node in
                let perfV2 = node.performanceV2
                let performance = GatewayNodePerformance(
                    lastUpdated: perfV2?.lastUpdatedUTC,
                    score: perfV2.map { mapScore(from: $0.score) } ?? .noScore,
                    mixnetScore: perfV2.map { mapScore(from: $0.mixnetScore) } ?? .noScore,
                    load: perfV2.map { mapScore(from: $0.load) } ?? .noScore,
                    uptime: perfV2?.uptimePercentageLast24Hours ?? 0
                )

                let asn = GatewayNodeASN(
                    asn: node.location.asn?.asn ?? "",
                    asnName: node.location.asn?.name ?? "",
                    type: node.location.asn.map { mapASNType(from: $0.kind) } ?? .other
                )

                var gatewayNodeLocation: GatewayNodeLocation?
                if let twoLetterIsoCountryCode = node.location.twoLetterISOCountryCode,
                   let latitude = node.location.latitude,
                   let longitude = node.location.longitude,
                   let city = node.location.city,
                   let region = node.location.region {
                    gatewayNodeLocation = GatewayNodeLocation(
                        twoLetterIsoCountryCode: twoLetterIsoCountryCode,
                        latitude: latitude,
                        longitude: longitude,
                        city: city,
                        region: region,
                        asn: asn
                    )
                }
                let bridges = mapBridgeInfo(from: node.bridges)

                return GatewayNode(
                    id: node.identityKey,
                    location: gatewayNodeLocation,
                    performance: performance,
                    mixnetScore: perfV2.map { mapScore(from: $0.score) } ?? .noScore,
                    name: node.name,
                    description: node.description,
                    buildVersion: node.buildInformation?.buildVersion,
                    ipv4s: node.ipAddresses?.ipv4s ?? [],
                    ipv6s: node.ipAddresses?.ipv6s ?? [],
                    bridges: bridges
                )
            }
        } catch {
            print("🔥🔥🔥 \(error)")
            return []
        }
    }

    func storeGatewayStore() {
#if SANTA
        appSettings.gatewayStore = GatewayCacheReloadPolicy.persistedGatewayStoreValue(for: gatewayStore)
#else
        Task { @MainActor in
            appSettings.gatewayStore = gatewayStore.rawValue
        }
#endif
    }

#if SANTA
    func clearGatewayStoreForEnvironmentChange() {
        entry = []
        exit = []
        vpn = []
        entryCountries = []
        exitCountries = []
        vpnCountries = []
        gatewayStore = GatewayNodeStore()
        appSettings.gatewayStore = nil
    }
#endif
}

private extension GatewayManager {
    func mapBridgeInfo(from bridges: Bridges?) -> GatewayBridgeInformation? {
        guard let newBridges = bridges
        else {
            return nil
        }

        let transports: [GatewayBridgeParameters] = newBridges.transports.compactMap { transport in
            switch transport.transportType {
            case .quicPlain:
                let args = transport.args
                let host: String? = args.host.isEmpty ? nil : args.host
                return .quicPlain(
                    GatewayQuicClientOptions(
                        addresses: args.addresses,
                        host: host,
                        idPubkey: args.idPubkey
                    )
                )
            }
        }
        return GatewayBridgeInformation(version: newBridges.version, transports: transports)
    }
}

private extension Array where Element == String {
    var ipv4s: [String] { filter { $0.contains(".") } }
    var ipv6s: [String] { filter { $0.contains(":") } }
}

private func mapScore(from load: Load?) -> GatewayNodeScore {
    switch load {
    case .high:
        .high
    case .medium:
        .medium
    case .low:
        .low
    case .offline:
        .offline
    case .none:
        .offline
    }
}

private func mapASNType(from kind: Kind) -> GatewayNodeASNType {
    switch kind {
    case .residential:
        .residential
    case .other:
        .other
    }
}

// MARK: - NodeElement
private struct NodeElement: Codable, Sendable {
    let identityKey: String
    let name: String
    let description: String?
    let ipPacketRouter: Authenticator?
    let authenticator: Authenticator?
    let location: Location
    let lastProbe: LastProbe?
    let ipAddresses: [String]?
    let mixPort: Int?
    let role: Role?
    let entry: Entry?
    let performance: String?
    let performanceV2: PerformanceV2?
    let mixnetScore: Load?
    let buildInformation: BuildInformation?
    let bridges: Bridges?

    enum CodingKeys: String, CodingKey {
        case identityKey = "identity_key"
        case name
        case description
        case ipPacketRouter = "ip_packet_router"
        case authenticator, location
        case lastProbe = "last_probe"
        case ipAddresses = "ip_addresses"
        case mixPort = "mix_port"
        case role, entry, performance
        case performanceV2 = "performance_v2"
        case buildInformation = "build_information"
        case mixnetScore = "mixnet_score"
        case bridges
    }
}

private struct Authenticator: Codable, Sendable { let address: String? }

// MARK: - BuildInformation
private struct BuildInformation: Codable, Sendable {
    let binaryName: String?
    let buildTimestamp: String?
    let buildVersion: String?
    let commitSHA: String?
    let commitTimestamp: String?
    let commitBranch: String?
    let rustcVersion: String?
    let rustcChannel: String?
    let cargoProfile: String?
    let cargoTriple: String?

    enum CodingKeys: String, CodingKey {
        case binaryName       = "binary_name"
        case buildTimestamp   = "build_timestamp"
        case buildVersion     = "build_version"
        case commitSHA        = "commit_sha"
        case commitTimestamp  = "commit_timestamp"
        case commitBranch     = "commit_branch"
        case rustcVersion     = "rustc_version"
        case rustcChannel     = "rustc_channel"
        case cargoProfile     = "cargo_profile"
        case cargoTriple      = "cargo_triple"
    }
}

// MARK: - Entry
private struct Entry: Codable, Sendable {
    let hostname: String?
    let wsPort: Int?
    let wssPort: Int?
    enum CodingKeys: String, CodingKey {
        case hostname
        case wsPort = "ws_port"
        case wssPort = "wss_port"
    }
}

// MARK: - LastProbe
private struct LastProbe: Codable, Sendable {
    let lastUpdatedUTC: Date?
    let outcome: Outcome?
    enum CodingKeys: String, CodingKey {
        case lastUpdatedUTC = "last_updated_utc"
        case outcome
    }
}

// MARK: - Outcome
private struct Outcome: Codable, Sendable {
    let asEntry: AsEntry?
    let asExit: AsExit?
    let wg: Wg?
    enum CodingKeys: String, CodingKey {
        case asEntry = "as_entry"
        case asExit = "as_exit"
        case wg
    }
}

private struct AsEntry: Codable, Sendable {
    let canConnect: Bool?
    let canRoute: Bool?
    enum CodingKeys: String, CodingKey {
        case canConnect = "can_connect"
        case canRoute = "can_route"
    }
}

private struct AsExit: Codable, Sendable {
    let canConnect: Bool?
    let canRouteIPV4: Bool?
    let canRouteIPExternalV4: Bool?
    let canRouteIPV6: Bool?
    let canRouteIPExternalV6: Bool?
    enum CodingKeys: String, CodingKey {
        case canConnect = "can_connect"
        case canRouteIPV4 = "can_route_ip_v4"
        case canRouteIPExternalV4 = "can_route_ip_external_v4"
        case canRouteIPV6 = "can_route_ip_v6"
        case canRouteIPExternalV6 = "can_route_ip_external_v6"
    }
}

// All metrics as Double? via lossy decoding to tolerate 0.5 etc.
private struct Wg: Codable, Sendable {
    let canRegister, canHandshake, canResolveDNS: Bool?
    let pingHostsPerformance: Double?
    let pingIPSPerformance: Double?
    let canHandshakeV4, canResolveDNSV4: Bool?
    let pingHostsPerformanceV4: Double?
    let pingIPSPerformanceV4: Double?
    let canHandshakeV6, canResolveDNSV6: Bool?
    let pingHostsPerformanceV6: Double?
    let pingIPSPerformanceV6: Double?
    let downloadDurationSECV4: Double?
    let downloadedFileV4: String?
    let downloadErrorV4: String?
    let downloadDurationSECV6: Double?
    let downloadedFileV6: String?
    let downloadErrorV6: String?

    enum CodingKeys: String, CodingKey {
        case canRegister = "can_register"
        case canHandshake = "can_handshake"
        case canResolveDNS = "can_resolve_dns"
        case pingHostsPerformance = "ping_hosts_performance"
        case pingIPSPerformance = "ping_ips_performance"
        case canHandshakeV4 = "can_handshake_v4"
        case canResolveDNSV4 = "can_resolve_dns_v4"
        case pingHostsPerformanceV4 = "ping_hosts_performance_v4"
        case pingIPSPerformanceV4 = "ping_ips_performance_v4"
        case canHandshakeV6 = "can_handshake_v6"
        case canResolveDNSV6 = "can_resolve_dns_v6"
        case pingHostsPerformanceV6 = "ping_hosts_performance_v6"
        case pingIPSPerformanceV6 = "ping_ips_performance_v6"
        case downloadDurationSECV4 = "download_duration_sec_v4"
        case downloadedFileV4 = "downloaded_file_v4"
        case downloadErrorV4 = "download_error_v4"
        case downloadDurationSECV6 = "download_duration_sec_v6"
        case downloadedFileV6 = "downloaded_file_v6"
        case downloadErrorV6 = "download_error_v6"
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        canRegister = try? container.decode(Bool.self, forKey: .canRegister)
        canHandshake = try? container.decode(Bool.self, forKey: .canHandshake)
        canResolveDNS = try? container.decode(Bool.self, forKey: .canResolveDNS)

        pingHostsPerformance = container.decodeLossyDouble(forKey: .pingHostsPerformance)
        pingIPSPerformance = container.decodeLossyDouble(forKey: .pingIPSPerformance)

        canHandshakeV4 = try? container.decode(Bool.self, forKey: .canHandshakeV4)
        canResolveDNSV4 = try? container.decode(Bool.self, forKey: .canResolveDNSV4)
        pingHostsPerformanceV4 = container.decodeLossyDouble(forKey: .pingHostsPerformanceV4)
        pingIPSPerformanceV4 = container.decodeLossyDouble(forKey: .pingIPSPerformanceV4)

        canHandshakeV6 = try? container.decode(Bool.self, forKey: .canHandshakeV6)
        canResolveDNSV6 = try? container.decode(Bool.self, forKey: .canResolveDNSV6)
        pingHostsPerformanceV6 = container.decodeLossyDouble(forKey: .pingHostsPerformanceV6)
        pingIPSPerformanceV6 = container.decodeLossyDouble(forKey: .pingIPSPerformanceV6)

        downloadDurationSECV4 = container.decodeLossyDouble(forKey: .downloadDurationSECV4)
        downloadedFileV4 = try? container.decode(String.self, forKey: .downloadedFileV4)
        downloadErrorV4 = try? container.decode(String.self, forKey: .downloadErrorV4)

        downloadDurationSECV6 = container.decodeLossyDouble(forKey: .downloadDurationSECV6)
        downloadedFileV6 = try? container.decode(String.self, forKey: .downloadedFileV6)
        downloadErrorV6 = try? container.decode(String.self, forKey: .downloadErrorV6)
    }
}

// MARK: - Bridges
private struct Bridges: Codable, Sendable {
    let version: String
    let transports: [Transport]
}

// MARK: - Transport
private struct Transport: Codable, Sendable {
    let transportType: TransportType
    let args: Args

    enum CodingKeys: String, CodingKey {
        case transportType = "transport_type"
        case args
    }
}

enum TransportType: String, Codable, Sendable {
    case quicPlain = "quic_plain"
}

// MARK: - Args
private struct Args: Codable, Sendable {
    let addresses: [String]
    let host: String
    let idPubkey: String

    enum CodingKeys: String, CodingKey {
        case addresses, host
        case idPubkey = "id_pubkey"
    }
}

// MARK: - Location
private struct Location: Codable, Sendable {
    let twoLetterISOCountryCode: String?
    let latitude, longitude: Double?
    let city, region, org, postal: String?
    let timezone: String?
    let asn: Asn?

    enum CodingKeys: String, CodingKey {
        case twoLetterISOCountryCode = "two_letter_iso_country_code"
        case latitude, longitude, city, region, org, postal, timezone, asn
    }
}

// MARK: - Asn
private struct Asn: Codable, Sendable {
    let asn, name, domain, route: String
    let kind: Kind
}

private enum Kind: String, Codable, Sendable { case other, residential }

// MARK: - PerformanceV2
private struct PerformanceV2: Codable, Sendable {
    let lastUpdatedUTC: Date
    let score, mixnetScore, load: Load?
    let uptimePercentageLast24Hours: Double

    enum CodingKeys: String, CodingKey {
        case lastUpdatedUTC = "last_updated_utc"
        case score, load
        case mixnetScore = "mixnet_score"
        case uptimePercentageLast24Hours = "uptime_percentage_last_24_hours"
    }
}

private extension KeyedDecodingContainer {
    func decodeLossyDouble(forKey key: K) -> Double? {
        if let value = try? decode(Double.self, forKey: key) {
            return value
        } else if let value = try? decode(Int.self, forKey: key) {
            return Double(value)
        } else if let value = try? decode(String.self, forKey: key) {
            return Double(value)
        } else {
            return nil
        }
    }

}

private enum Load: String, Codable, Sendable {
    case high, low, medium, offline
}

private enum Role: String, Codable, Sendable {
    case entryGateway = "EntryGateway",
         exitGateway = "ExitGateway",
         inactive = "Inactive"
}

private typealias Node = [NodeElement]
