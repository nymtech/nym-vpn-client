import Foundation

public final class MixnetTunnelProvider: OsTunProvider {
    public var nymOnConfigure: (() -> Void)?
    public var fileDescriptor: Int32?
    public var nymConfig: NymConfig?

    public init(nymOnConfigure: (() -> Void)? = nil, fileDescriptor: Int32? = nil) {
        self.nymOnConfigure = nymOnConfigure
        self.fileDescriptor = fileDescriptor
    }

    public func configureNym(config: NymConfig) throws -> Int32 {
        let semaphore = DispatchSemaphore(value: 0)
        nymConfig = config

        guard let nymOnConfigure else { throw FfiError.FdNotFound }

        nymOnConfigure()
        semaphore.signal()

        semaphore.wait()

        guard let fileDescriptor else { throw FfiError.FdNotFound }
        return Int32(fileDescriptor)
    }

    public func configureWg(config: WgConfig) throws {
//        let newConfiguration = tunnelConfiguration(from: config)
//        print(tunnelConfiguration)
//        appSettings.tunnelConfiguration = newConfiguration
    }
}

// private extension MixnetTunnelProvider {
//    func tunnelConfiguration(from config: WgConfig) -> TunnelConfiguration? {
//        guard let privateKey = WireGuardKit.PrivateKey(base64Key: config.tunnel.privateKey)
//        else {
//            return nil
//        }
//
//        var interfaceConfiguration = InterfaceConfiguration(privateKey: privateKey)
//        if let ipv6Gateway = config.ipv6Gateway, let ipv6AddressRange = IPAddressRange(from: ipv6Gateway) {
//            interfaceConfiguration.addresses.append(ipv6AddressRange)
//        }
//        interfaceConfiguration.addresses = [
//            IPAddressRange(from: config.ipv4Gateway)
//        ].compactMap { $0 }
//
//        let peers = config.peers.compactMap { configuration -> PeerConfiguration? in
//            guard
//                let publicKey = WireGuardKit.PublicKey(base64Key: configuration.publicKey),
//                let psk = configuration.psk,
//                let presharedKey = WireGuardKit.PreSharedKey(base64Key: psk)
//            else {
//                return nil
//            }
//
//            var peerConfiguration = WireGuardKit.PeerConfiguration(publicKey: publicKey)
//            peerConfiguration.preSharedKey = presharedKey
//            peerConfiguration.allowedIPs = configuration.allowedIps.compactMap {
//                IPAddressRange(from: $0)
//            }
//            peerConfiguration.endpoint = Endpoint(from: configuration.endpoint)
//            return peerConfiguration
//        }
//
//        return TunnelConfiguration(
//            name: "NymVPN 2 Hop",
//            interface: interfaceConfiguration,
//            peers: peers
//        )
//    }
// }
