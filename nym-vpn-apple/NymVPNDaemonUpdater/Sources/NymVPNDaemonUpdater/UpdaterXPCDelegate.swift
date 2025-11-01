import Foundation
import Security
import NymVPNDaemonUpdaterProtocol

final class UpdaterXPCDelegate: NSObject, NSXPCListenerDelegate {
    func listener(_ listener: NSXPCListener, shouldAcceptNewConnection connection: NSXPCConnection) -> Bool {
        guard isClientAuthorized(pid: connection.processIdentifier)
        else {
            NSLog("❌ Unauthorized connection attempt from PID \(connection.processIdentifier)")
            return false
        }

        connection.exportedInterface = NSXPCInterface(with: UpdaterProtocol.self)
        connection.exportedObject = UpdaterHandler()
        connection.resume()
        return true
    }

    // MARK: - Authorization

    private func isClientAuthorized(pid: pid_t) -> Bool {
        var attributes: [CFString: Any] = [kSecGuestAttributePid: pid]
        var clientCode: SecCode?

        let guestStatus = SecCodeCopyGuestWithAttributes(nil, attributes as CFDictionary, [], &clientCode)
        guard guestStatus == errSecSuccess, let clientCode
        else {
            NSLog("⚠️ Failed to obtain SecCode for PID \(pid): \(guestStatus)")
            return false
        }

        // --- Basic signing info ---
        var infoRef: CFDictionary?

        var staticCodeOpt: SecStaticCode?
        let err = SecCodeCopyStaticCode(clientCode, [], &staticCodeOpt)
        guard err == errSecSuccess,
              let staticCode = staticCodeOpt
        else {
            return false
        }

        let infoStatus = SecCodeCopySigningInformation(
            staticCode,
            SecCSFlags(rawValue: kSecCSSigningInformation),
            &infoRef
        )
        guard infoStatus == errSecSuccess,
              let info = infoRef as? [CFString: Any],
              let identifier = info[kSecCodeInfoIdentifier] as? String,
              let teamID = info[kSecCodeInfoTeamIdentifier] as? String
        else {
            NSLog("⚠️ Missing signing info or identifiers")
            return false
        }

        // Expected identifiers
        let expectedIdentifier = "net.nymtech.vpn"
        let expectedTeamID = "VW5DZLFHM5"

        guard identifier == expectedIdentifier,
            teamID == expectedTeamID
        else {
            NSLog("❌ Rejected client: id=\(identifier), team=\(teamID)")
            return false
        }

        NSLog("✅ Authorized connection from \(identifier) [Team \(teamID)]")
        return true
    }
}
