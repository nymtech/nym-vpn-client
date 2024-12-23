// swiftlint:disable:this file_name
//
//  TunnelSettingsConversions.swift
//  NymMixnetTunnel
//
//  Created by pronebird on 29/8/24.
//

import Foundation
import NetworkExtension
import MixnetLibrary
import TunnelMixnet

extension TunnelNetworkSettings {
    func asPacketTunnelNetworkSettings() -> NEPacketTunnelNetworkSettings {
        let networkSettings = NEPacketTunnelNetworkSettings(tunnelRemoteAddress: tunnelRemoteAddress)
        networkSettings.ipv4Settings = ipv4Settings?.asNEIPv4Settings()
        networkSettings.ipv6Settings = ipv6Settings?.asNEIPv6Settings()
        networkSettings.mtu = NSNumber(value: mtu)
        networkSettings.dnsSettings = dnsSettings?.asNEDNSSettings()

        return networkSettings
    }
}

extension DnsSettings {
    func asNEDNSSettings() -> NEDNSSettings {
        let dnsSettings = NEDNSSettings(servers: servers)
        dnsSettings.searchDomains = searchDomains
        dnsSettings.matchDomains = matchDomains
        return dnsSettings
    }
}

extension Ipv4Settings {
    func asNEIPv4Settings() -> NEIPv4Settings {
        var addresses = [String]()
        var netmasks = [String]()

        for address in self.addresses {
            if let addrRange = IPAddressRange(from: address) {
                if let ipv4Addr = addrRange.address as? IPv4Address {
                    addresses.append("\(ipv4Addr)")
                    netmasks.append("\(addrRange.subnetMask())")
                }
            }
        }

        let ipv4Settings = NEIPv4Settings(addresses: addresses, subnetMasks: netmasks)
        ipv4Settings.includedRoutes = includedRoutes?.map { $0.asNEIPv4Route() }
        ipv4Settings.excludedRoutes = excludedRoutes?.map { $0.asNEIPv4Route() }

        return ipv4Settings
    }
}

extension Ipv6Settings {
    func asNEIPv6Settings() -> NEIPv6Settings {
        var addresses = [String]()
        var networkPrefixes = [NSNumber]()

        for address in self.addresses {
            if let addrRange = IPAddressRange(from: address) {
                if let ipv6Addr = addrRange.address as? IPv6Address {
                    addresses.append("\(ipv6Addr)")
                    networkPrefixes.append(NSNumber(value: addrRange.networkPrefixLength))
                }
            }
        }

        let ipv6Settings = NEIPv6Settings(addresses: addresses, networkPrefixLengths: networkPrefixes)
        ipv6Settings.includedRoutes = includedRoutes?.map { $0.asNEIPv6Route() }
        ipv6Settings.excludedRoutes = excludedRoutes?.map { $0.asNEIPv6Route() }
        return ipv6Settings
    }
}

extension Ipv4Route {
    func asNEIPv4Route() -> NEIPv4Route {
        switch self {
        case .default:
            return NEIPv4Route.default()

        case let .specific(destination, subnetMask, gateway):
            let ipv4Route = NEIPv4Route(destinationAddress: destination, subnetMask: subnetMask)
            ipv4Route.gatewayAddress = gateway
            return ipv4Route
        }
    }
}

extension Ipv6Route {
    func asNEIPv6Route() -> NEIPv6Route {
        switch self {
        case .default:
            return NEIPv6Route.default()

        case let .specific(destination, prefixLength, gateway):
            let ipv6Route = NEIPv6Route(
                destinationAddress: destination,
                networkPrefixLength: NSNumber(value: prefixLength)
            )
            ipv6Route.gatewayAddress = gateway
            return ipv6Route
        }
    }
}
