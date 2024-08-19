#if os(iOS)
import Foundation
import Constants
import NetworkExtension
import MixnetLibrary

public final class MixnetAdapter {
    private weak var packetTunnelProvider: NEPacketTunnelProvider?

    public let mixnetTunnelProvider: MixnetTunnelProvider

    public var tunnelFileDescriptor: Int32? {
        var buf = [CChar](repeating: 0, count: Int(IFNAMSIZ))
        for fd: Int32 in 0...1024 {
            var len = socklen_t(buf.count)
            if getsockopt(fd, 2, 2, &buf, &len) == 0 && String(cString: buf).hasPrefix("utun") {
                return fd
            }
        }
        return nil
    }

    public init(
        with packetTunnelProvider: NEPacketTunnelProvider,
        mixnetTunnelProvider: MixnetTunnelProvider
    ) {
        self.packetTunnelProvider = packetTunnelProvider
        self.mixnetTunnelProvider = mixnetTunnelProvider
    }

    public func start(with vpnConfig: VpnConfig) throws {
        do {
            try runVpn(config: vpnConfig)
        } catch let error {
            throw error
        }
    }

    public func stop() throws {
        do {
            try stopVpn()
        } catch let error {
            throw error
        }
    }
}
#endif
