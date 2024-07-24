import NetworkExtension
import Logging
import WireGuardKit
import Tunnels
import TunnelWG

class PacketTunnelProvider: NEPacketTunnelProvider {

    private lazy var logger = Logger(label: "WireGuardAdapter")

    private lazy var adapter: WireGuardAdapter = {
        let wireguardLogger = Logger(label: "WireGuardAdapter")
        return WireGuardAdapter(
            with: self,
            logHandler: { level, message in
                wireguardLogger.log(level: level.loggerLevel, "\(message)")
            }
        )
    }()

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        logger.log(level: .info, "Starting tunnel...")

        guard
            let tunnelProviderProtocol = self.protocolConfiguration as? NETunnelProviderProtocol,
            let tunnelConfiguration = tunnelProviderProtocol.asTunnelConfiguration()
        else {
            completionHandler(PacketTunnelProviderError.invalidSavedConfiguration)
            return
        }

        adapter.start(tunnelConfiguration: tunnelConfiguration) { [weak self] adapterError in
            guard let adapterError = adapterError
            else {
                let interfaceName = self?.adapter.interfaceName ?? "unknown"
                self?.logger.log(level: .info, "Tunnel interface: \(interfaceName)")
                completionHandler(nil)
                return
            }

            self?.handleError(with: adapterError, completionHandler: completionHandler)
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        adapter.stop { [weak self] error in
            if let error = error {
                self?.logger.log(level: .error, "Failed to stop adapter: \(error.localizedDescription)")
            }
            completionHandler()
        }
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)? = nil) {
        guard let completionHandler, !messageData.isEmpty else { return }

        adapter.getRuntimeConfiguration { settings in
            completionHandler(settings?.data(using: .utf8))
        }
    }
}

private extension PacketTunnelProvider {
    func handleError(with adapterError: WireGuardAdapterError, completionHandler: @escaping (Error?) -> Void) {
        switch adapterError {
        case .cannotLocateTunnelFileDescriptor:
            logger.log(level: .error, "Starting tunnel failed: could not determine file descriptor")
            completionHandler(PacketTunnelProviderError.fileDescriptorFailure)
        case .dnsResolution(let dnsErrors):
            let hostnamesWithDnsResolutionFailure = dnsErrors.map { $0.address } .joined(separator: ", ")
            logger.log(level: .error, "DNS resolution failed for the following hostnames: \(hostnamesWithDnsResolutionFailure)")
            completionHandler(PacketTunnelProviderError.dnsResolveFailure)
        case .setNetworkSettings(let error):
            logger.log(level: .error, "Starting tunnel failed with setTunnelNetworkSettings returning \(error.localizedDescription)")
            completionHandler(PacketTunnelProviderError.saveNetworkSettingsFailure)
        case .startWireGuardBackend(let errorCode):
            logger.log(level: .error, "Starting tunnel failed with wgTurnOn returning \(errorCode)")
            completionHandler(PacketTunnelProviderError.backendStartFailure)
        case .invalidState:
            // Must never happen
            fatalError("Invalid Packet Tunnel Provider")
        }
    }
}
