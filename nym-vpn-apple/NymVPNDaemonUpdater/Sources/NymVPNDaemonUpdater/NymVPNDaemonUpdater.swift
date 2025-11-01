import Foundation

@main
struct NymVPNDaemonUpdater {
    static func main() {
        let listener = NSXPCListener(machServiceName: "net.nymtech.vpn.updater.xpc")
        let delegate = UpdaterXPCDelegate()
        listener.delegate = delegate
        listener.resume()

        RunLoop.current.run()
    }
}
