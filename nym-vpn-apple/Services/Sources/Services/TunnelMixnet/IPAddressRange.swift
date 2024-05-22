import Foundation
import Network

public struct IPAddressRange {
    public let address: IPAddress
    public let networkPrefixLength: UInt8
}

extension IPAddressRange: Equatable {
    public static func == (lhs: IPAddressRange, rhs: IPAddressRange) -> Bool {
        lhs.address.rawValue == rhs.address.rawValue && lhs.networkPrefixLength == rhs.networkPrefixLength
    }
}

extension IPAddressRange: Hashable {
    public func hash(into hasher: inout Hasher) {
        hasher.combine(address.rawValue)
        hasher.combine(networkPrefixLength)
    }
}

extension IPAddressRange {
    public var stringRepresentation: String {
        "\(address)/\(networkPrefixLength)"
    }

    public init?(from string: String) {
        guard let parsed = IPAddressRange.parseAddressString(string) else { return nil }
        address = parsed.0
        networkPrefixLength = parsed.1
    }

    private static func parseAddressString(_ string: String) -> (IPAddress, UInt8)? {
        let endOfIPAddress = string.lastIndex(of: "/") ?? string.endIndex
        let addressString = String(string[string.startIndex ..< endOfIPAddress])
        let address: IPAddress
        if let addr = IPv4Address(addressString) {
            address = addr
        } else if let addr = IPv6Address(addressString) {
            address = addr
        } else {
            return nil
        }

        let maxNetworkPrefixLength: UInt8 = address is IPv4Address ? 32 : 128
        var networkPrefixLength: UInt8
        if endOfIPAddress < string.endIndex { // "/" was located
            let indexOfNetworkPrefixLength = string.index(after: endOfIPAddress)
            guard indexOfNetworkPrefixLength < string.endIndex else { return nil }
            let networkPrefixLengthSubstring = string[indexOfNetworkPrefixLength ..< string.endIndex]
            guard let npl = UInt8(networkPrefixLengthSubstring) else { return nil }
            networkPrefixLength = min(npl, maxNetworkPrefixLength)
        } else {
            networkPrefixLength = maxNetworkPrefixLength
        }

        return (address, networkPrefixLength)
    }

    public func subnetMask() -> IPAddress {
        if address is IPv4Address {
            let mask = networkPrefixLength > 0 ? ~UInt32(0) << (32 - networkPrefixLength) : UInt32(0)
            let bytes = Data(
                [
                    UInt8(truncatingIfNeeded: mask >> 24),
                    UInt8(truncatingIfNeeded: mask >> 16),
                    UInt8(truncatingIfNeeded: mask >> 8),
                    UInt8(truncatingIfNeeded: mask >> 0)
                ]
            )
            guard let address = IPv4Address(bytes)
            else {
                fatalError("Cannot retrieve ipv4 subnetMask IP Address")
            }
            return address
        }
        if address is IPv6Address {
            var bytes = Data(repeating: 0, count: 16)
            for index in 0..<Int(networkPrefixLength / 8) {
                bytes[index] = 0xff
            }
            let nibble = networkPrefixLength % 32
            if nibble != 0 {
                let mask = ~UInt32(0) << (32 - nibble)
                let index = Int(networkPrefixLength / 32 * 4)
                bytes[index + 0] = UInt8(truncatingIfNeeded: mask >> 24)
                bytes[index + 1] = UInt8(truncatingIfNeeded: mask >> 16)
                bytes[index + 2] = UInt8(truncatingIfNeeded: mask >> 8)
                bytes[index + 3] = UInt8(truncatingIfNeeded: mask >> 0)
            }
            guard let address = IPv6Address(bytes)
            else {
                fatalError("Cannot retrieve ipv6 subnetMask IP Address")
            }
            return address
        }
        fatalError("Cannot retrieve subnetMask IP Address")
    }

    public func maskedAddress() -> IPAddress {
        let subnet = subnetMask().rawValue
        var masked = Data(address.rawValue)
        if subnet.count != masked.count {
            fatalError("Cannot retrieve maskedAddress IP Address")
        }
        for index in 0..<subnet.count {
            masked[index] &= subnet[index]
        }
        if subnet.count == 4 {
            guard let address = IPv4Address(masked)
            else {
                fatalError("Cannot retrieve ipv4 maskedAddress IP Address")
            }
            return address
        }
        if subnet.count == 16 {
            guard let address = IPv6Address(masked)
            else {
                fatalError("Cannot retrieve ipv6 maskedAddress IP Address")
            }
            return address
        }
        fatalError("Cannot retrieve maskedAddress IP Address")
    }
}
