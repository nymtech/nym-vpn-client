#if os(iOS)
import NetworkExtension
import MixnetLibrary

public final class MixnetTunnelSettingsGenerator {
    private let nymConfig: NymConfig

    public init(nymConfig: NymConfig) {
        self.nymConfig = nymConfig
    }

    public func generateNetworkSettings() -> NEPacketTunnelNetworkSettings {
        // iOS requires a tunnel endpoint, whereas in WireGuard it's valid for
        // a tunnel to have no endpoint, or for there to be many endpoints, in
        // which case, displaying a single one in settings doesn't really
        // make sense. So, we fill it in with this placeholder, which is not
        // a valid IP address that will actually route over the Internet.
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: "127.0.0.1")

        let mtu = nymConfig.mtu

        // 0 means automatic MTU. In theory, we should just do
        // `networkSettings.tunnelOverheadBytes = 80` but in
        // practice there are too many broken networks out there.
        // Instead set it to 1280. Boohoo. Maybe someday we'll
        // add a nob, maybe, or iOS will do probing for us.
        if mtu == 0 {
        #if os(iOS)
            networkSettings.mtu = NSNumber(value: 1280)
        #elseif os(macOS)
            networkSettings.tunnelOverheadBytes = 80
        #else
            #error("Unimplemented")
        #endif
        } else {
            networkSettings.mtu = NSNumber(value: mtu)
        }

        let (ipv4Addresses, ipv6Addresses) = addresses()
        let (ipv4IncludedRoutes, ipv6IncludedRoutes) = includedRoutes()
        let (ipv4ExcludedRoutes, ipv6ExcludedRoutes) = excludedRoutes()

        let ipv4Settings = NEIPv4Settings(
            addresses: ipv4Addresses.map { $0.destinationAddress },
            subnetMasks: ipv4Addresses.map { $0.destinationSubnetMask }
        )
        ipv4Settings.includedRoutes = ipv4IncludedRoutes
        ipv4Settings.excludedRoutes = ipv4ExcludedRoutes
        networkSettings.ipv4Settings = ipv4Settings

        let ipv6Settings = NEIPv6Settings(
            addresses: ipv6Addresses.map { $0.destinationAddress },
            networkPrefixLengths: ipv6Addresses.map { $0.destinationNetworkPrefixLength }
        )
        ipv6Settings.includedRoutes = ipv6IncludedRoutes
        ipv6Settings.excludedRoutes = ipv6ExcludedRoutes
        networkSettings.ipv6Settings = ipv6Settings

        // TODO: expose DNS settings in uniffy layer and set the values here
        networkSettings.dnsSettings = NEDNSSettings(servers: ["1.1.1.1", "1.0.0.1"])

        return networkSettings
    }

    private func addresses() -> ([NEIPv4Route], [NEIPv6Route]) {
        var ipv4Routes = [NEIPv4Route]()
        var ipv6Routes = [NEIPv6Route]()
        let ipv4AddressRange = IPAddressRange(from: nymConfig.ipv4Addr)
        let ipv6AddressRange = IPAddressRange(from: nymConfig.ipv6Addr)

        if let addressRange = ipv4AddressRange, addressRange.address is IPv4Address {
            ipv4Routes.append(
                NEIPv4Route(
                    destinationAddress: "\(addressRange.address)",
                    subnetMask: "\(addressRange.subnetMask())"
                )
            )
        }

        if let addressRange = ipv6AddressRange, addressRange.address is IPv6Address {
            // Big fat ugly hack for broken iOS networking stack: the smallest prefix that will have
            // any effect on iOS is a /120, so we clamp everything above to /120. This is potentially
            // very bad, if various network parameters were actually relying on that subnet being
            // intentionally small. TODO: talk about this with upstream iOS devs.
            ipv6Routes.append(NEIPv6Route(destinationAddress: "\(addressRange.address)", networkPrefixLength: NSNumber(value: min(120, addressRange.networkPrefixLength))))
        }
        return (ipv4Routes, ipv6Routes)
    }

    private func includedRoutes() -> ([NEIPv4Route], [NEIPv6Route]) {
        var ipv4IncludedRoutes = [NEIPv4Route]()
        var ipv6IncludedRoutes = [NEIPv6Route]()

        let ipv4AddressRange = IPAddressRange(from: nymConfig.ipv4Addr)
        let ipv6AddressRange = IPAddressRange(from: nymConfig.ipv6Addr)

        let ipv4includedRouteRange = IPAddressRange(from: "0.0.0.0/0")
        let ipv6includedRouteRange = IPAddressRange(from: "::/0")

        if let addressRange = ipv4AddressRange, addressRange.address is IPv4Address {
            let route = NEIPv4Route(
                destinationAddress: "\(addressRange.maskedAddress())",
                subnetMask: "\(addressRange.subnetMask())"
            )
            route.gatewayAddress = "\(addressRange.address)"
            ipv4IncludedRoutes.append(route)
        }

        if let addressRange = ipv6AddressRange, addressRange.address is IPv6Address {
            let route = NEIPv6Route(
                destinationAddress: "\(addressRange.maskedAddress())",
                networkPrefixLength: NSNumber(value: addressRange.networkPrefixLength)
            )
            route.gatewayAddress = "\(addressRange.address)"
            ipv6IncludedRoutes.append(route)
        }

        if let addressRange = ipv4includedRouteRange {
            ipv4IncludedRoutes.append(
                NEIPv4Route(
                    destinationAddress: "\(addressRange.address)",
                    subnetMask: "\(addressRange.subnetMask())"
                )
            )
        }

        if let addressRange = ipv6includedRouteRange {
            ipv6IncludedRoutes.append(
                NEIPv6Route(
                    destinationAddress: "\(addressRange.address)",
                    networkPrefixLength: NSNumber(value: addressRange.networkPrefixLength)
                )
            )
        }

        return (ipv4IncludedRoutes, ipv6IncludedRoutes)
    }

    private func excludedRoutes() -> ([NEIPv4Route], [NEIPv6Route]) {
        var ipv4ExcludedRoutes = [NEIPv4Route]()
        var ipv6ExcludedRoutes = [NEIPv6Route]()

        guard
            let entryMixnetGatewayIp = nymConfig.entryMixnetGatewayIp,
            let addressRange = IPAddressRange(from: entryMixnetGatewayIp)
        else {
            return (ipv4ExcludedRoutes, ipv6ExcludedRoutes)
        }

        if addressRange.address is IPv4Address {
            ipv4ExcludedRoutes.append(
                NEIPv4Route(
                    destinationAddress: "\(addressRange.address)",
                    subnetMask: "\(addressRange.subnetMask())"
                )
            )
        }

        if addressRange.address is IPv6Address {
            ipv6ExcludedRoutes.append(
                NEIPv6Route(
                    destinationAddress: "\(addressRange.address)",
                    networkPrefixLength: NSNumber(value: addressRange.networkPrefixLength)
                )
            )
        }
        return (ipv4ExcludedRoutes, ipv6ExcludedRoutes)
    }
}
#endif
