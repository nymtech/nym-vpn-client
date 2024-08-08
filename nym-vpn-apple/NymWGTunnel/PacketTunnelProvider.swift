import NetworkExtension
import Logging
import Tunnels
import TunnelWG
import MixnetLibrary

class PacketTunnelProvider: NEPacketTunnelProvider {

    private lazy var logger = Logger(label: "PacketTunnelProvider")

    override func startTunnel(options: [String: NSObject]?, completionHandler: @escaping (Error?) -> Void) {
        logger.log(level: .info, "Starting tunnel...")

        guard
            let tunnelProviderProtocol = self.protocolConfiguration as? NETunnelProviderProtocol,
            let tunnelConfiguration = tunnelProviderProtocol.asTunnelConfiguration()
        else {
            completionHandler(PacketTunnelProviderError.invalidSavedConfiguration)
            return
        }

        let vpnConfig = VpnConfig(
            apiUrl: <#T##Url#>,
            explorerUrl: <#T##Url#>,
            entryGateway: <#T##EntryPoint#>,
            exitRouter: <#T##ExitPoint#>,
            enableTwoHop: <#T##Bool#>,
            tunProvider: <#T##any OsTunProvider#>,
            credentialDataPath: <#T##PathBuf?#>,
            tunStatusListener: <#T##(any TunnelStatusListener)?#>
        )

        completionHandler(nil)

        DispatchQueue.global().async {
            do {
                try runVpn(config: vpnConfig)
            } catch {

            }
        }
    }

    override func stopTunnel(with reason: NEProviderStopReason, completionHandler: @escaping () -> Void) {
        try? stopVpn()
        completionHandler()
    }

    override func handleAppMessage(_ messageData: Data, completionHandler: ((Data?) -> Void)? = nil) {
        completionHandler?(nil)
    }
}


extension PacketTunnelProvider: OsTunProvider {
    func configureWg(config: MixnetLibrary.WgConfig) throws {

    }
    
    func configureNym(config: MixnetLibrary.NymConfig) throws -> Int32 {

    }
    

}
