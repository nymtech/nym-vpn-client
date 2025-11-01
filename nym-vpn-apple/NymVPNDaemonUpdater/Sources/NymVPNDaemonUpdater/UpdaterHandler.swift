import Foundation
import NymVPNDaemonUpdaterProtocol

final class UpdaterHandler: NSObject, UpdaterProtocol {
    func killHelper(completion: @escaping (Bool, String?) -> Void) {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/bin/sh")
        process.arguments = ["-c", "pgrep net.nymtech.vpn.helper | xargs kill -9"]

        let pipe = Pipe()
        process.standardError = pipe
        process.standardOutput = pipe

        do {
            try process.run()
            process.waitUntilExit()
            let data = pipe.fileHandleForReading.readDataToEndOfFile()
            let output = String(data: data, encoding: .utf8) ?? ""
            completion(process.terminationStatus == 0, output.isEmpty ? nil : output)
        } catch {
            completion(false, error.localizedDescription)
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
            exit(EXIT_SUCCESS)
        }
    }
}
