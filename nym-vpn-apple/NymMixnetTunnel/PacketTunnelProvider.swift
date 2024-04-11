import NetworkExtension
import TunnelMixnet
import Tunnels
import MixnetLibrary

class PacketTunnelProvider: NEPacketTunnelProvider {
    private lazy var mixnetTunnelProvider = MixnetTunnelProvider()
    private lazy var mixnetAdapter: MixnetAdapter = {
        return MixnetAdapter(
            with: self,
            mixnetTunnelProvider: mixnetTunnelProvider
        )
    }()

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        guard
            let tunnelProviderProtocol = self.protocolConfiguration as? NETunnelProviderProtocol,
            let mixnetConfig = tunnelProviderProtocol.asMixnetConfig()
        else {
            completionHandler(PacketTunnelProviderError.invalidSavedConfiguration)
            return
        }

        let semaphore = DispatchSemaphore(value: 0)

        let callback: () -> Void = { [weak self] in
            guard let config = self?.mixnetTunnelProvider.nymConfig
            else {
                semaphore.signal()
                return
            }

            self?.configure(with: config)
            self?.mixnetTunnelProvider.fileDescriptor = self?.mixnetAdapter.tunnelFileDescriptor
            semaphore.signal()
        }
        mixnetTunnelProvider.nymOnConfigure = callback
        do {
            try mixnetAdapter.start(
                with: mixnetConfig.asVpnConfig(mixnetTunnelProvider: mixnetAdapter.mixnetTunnelProvider)
            )
        } catch let error {
            completionHandler(error)
        }
        semaphore.wait()

        completionHandler(nil)
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        do {
            try mixnetAdapter.stop()
        } catch let error {
            // TODO: handle error
            print(error)
        }
        completionHandler()
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)?) {
        // Add code here to handle the message.
        if let handler = completionHandler {
            handler(messageData)
        }
    }

    override func sleep(completionHandler: @escaping () -> Void) {
        // Add code here to get ready to sleep.
        completionHandler()
    }

    override func wake() {
        // Add code here to wake up.
    }
}

private extension PacketTunnelProvider {
    func configure(with config: NymConfig) {
        let networkSettings = MixnetTunnelSettingsGenerator(nymConfig: config).generateNetworkSettings()
        do {
            try? setNetworkSettings(networkSettings)
        }
    }

    /// Set network tunnel configuration.
    /// This method ensures that the call to `setTunnelNetworkSettings` does not time out, as in
    /// certain scenarios the completion handler given to it may not be invoked by the system.
    ///
    /// - Parameters:
    ///   - networkSettings: an instance of type `NEPacketTunnelNetworkSettings`.
    /// - Throws: an error of type `WireGuardAdapterError`.
    /// - Returns: `PacketTunnelSettingsGenerator`.
    private func setNetworkSettings(_ networkSettings: NEPacketTunnelNetworkSettings) throws {
        var systemError: Error?
        let condition = NSCondition()

        // Activate the condition
        condition.lock()
        defer { condition.unlock() }

        setTunnelNetworkSettings(networkSettings) { error in
            systemError = error
            condition.signal()
        }

        // Packet tunnel's `setTunnelNetworkSettings` times out in certain
        // scenarios & never calls the given callback.
        let setTunnelNetworkSettingsTimeout: TimeInterval = 5 // seconds

        if condition.wait(until: Date().addingTimeInterval(setTunnelNetworkSettingsTimeout)) {
            // TODO: handle error
            if let systemError = systemError {
                // throw WireGuardAdapterError.setNetworkSettings(systemError)
            }
        } else {
            // self.logHandler(.error, "setTunnelNetworkSettings timed out after 5 seconds; proceeding anyway")
        }
    }
}
