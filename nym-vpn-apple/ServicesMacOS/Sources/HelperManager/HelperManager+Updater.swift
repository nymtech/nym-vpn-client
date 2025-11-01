import ServiceManagement
import NymVPNDaemonUpdaterProtocol

extension HelperManager {
    func callKillHelper() {
        let connection = NSXPCConnection(machServiceName: "net.nymtech.vpn.updater.xpc", options: [])
        connection.remoteObjectInterface = NSXPCInterface(with: UpdaterProtocol.self)
        connection.resume()

        let proxy = connection.remoteObjectProxyWithErrorHandler { [weak self] error in
            self?.logger.error("❌ XPC error: \(error)")
        } as? UpdaterProtocol

        proxy?.killHelper { [weak self] success, message in
            if success {
                self?.logger.info("✅ net.nymtech.vpn.helper terminated")
            } else {
                self?.logger.error("❌ Failed to kill helper: \(message ?? "unknown")")
            }
            connection.invalidate()
        }
    }
}
