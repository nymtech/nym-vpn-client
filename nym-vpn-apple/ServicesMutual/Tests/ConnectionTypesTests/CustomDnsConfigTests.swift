import Foundation
import NymVPNLib
import Testing
@testable import ConnectionTypes

@Suite struct CustomDnsConfigTests {
    static let allServers: [CustomDnsConfig.Server] = [
        .plain(address: "1.1.1.1", port: nil, ipProto: .udp),
        .plain(address: "1.0.0.1", port: 5353, ipProto: .tcp),
        .plain(address: "2606:4700:4700::1111", port: nil, ipProto: .udpAndTcp),
        .tls(address: "9.9.9.9", port: 853, serverName: "dns.quad9.net"),
        .https(address: "8.8.8.8", port: nil, serverName: "dns.google", queryPath: "/dns-query"),
        .quic(address: "94.140.14.14", port: nil, serverName: "dns.adguard-dns.com"),
        .http3(address: "1.1.1.1", port: 443, serverName: "cloudflare-dns.com", queryPath: nil)
    ]

    static func makeConnectionConfig(customDns: CustomDnsConfig) -> ConnectionConfig {
        ConnectionConfig(
            entry: .random,
            exit: .random,
            allowLan: false,
            disableIpv6: false,
            enableTwoHop: true,
            bridgeMode: .off,
            enableAdBlocking: false,
            netstack: false,
            residentialExit: false,
            mixnetTuningConfig: MixnetTuningConfig(
                backgroundTraffic: .ms200,
                continuousTraffic: .ms20,
                dissablePoissonRate: false
            ),
            splitTunnelConfig: SplitTunnelConfig(),
            customDns: customDns
        )
    }

    @Test func settingsRoundTrip() {
        let config = CustomDnsConfig(isEnabled: true, dnsServers: Self.allServers)
        #expect(CustomDnsConfig(from: config.settings) == config)
    }

    @Test func connectionConfigJsonRoundTrip() throws {
        let config = Self.makeConnectionConfig(
            customDns: CustomDnsConfig(isEnabled: true, dnsServers: Self.allServers)
        )
        let json = try #require(config.toJson())
        #expect(ConnectionConfig.from(jsonString: json) == config)
    }

    @Test func decodesLegacyDnsAddresses() throws {
        let json = try #require(Self.makeConnectionConfig(customDns: CustomDnsConfig()).toJson())
        var object = try #require(
            try JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any]
        )
        object.removeValue(forKey: "customDns")
        object["dns"] = ["1.1.1.1", "9.9.9.9"]
        let legacyJson = try #require(
            String(data: try JSONSerialization.data(withJSONObject: object), encoding: .utf8)
        )

        let decoded = try #require(ConnectionConfig.from(jsonString: legacyJson))
        #expect(decoded.customDns == CustomDnsConfig(
            isEnabled: true,
            dnsServers: [
                .plain(address: "1.1.1.1", port: nil, ipProto: .udpAndTcp),
                .plain(address: "9.9.9.9", port: nil, ipProto: .udpAndTcp)
            ]
        ))
    }

    @Test func decodesMissingDnsAsDisabled() throws {
        let json = try #require(Self.makeConnectionConfig(customDns: CustomDnsConfig()).toJson())
        var object = try #require(
            try JSONSerialization.jsonObject(with: Data(json.utf8)) as? [String: Any]
        )
        object.removeValue(forKey: "customDns")
        let legacyJson = try #require(
            String(data: try JSONSerialization.data(withJSONObject: object), encoding: .utf8)
        )

        let decoded = try #require(ConnectionConfig.from(jsonString: legacyJson))
        #expect(decoded.customDns == CustomDnsConfig())
    }
}
