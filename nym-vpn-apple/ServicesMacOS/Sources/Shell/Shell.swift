import Foundation

public struct Shell {
    // Required so we could detect SMJobBless apps and suggest a migration.
    // This one can be deprecated, once more people update to SMAppService.
    @discardableResult public static func exec(command: String) -> String? {
        let task = Process()
        task.executableURL = URL(fileURLWithPath: "/bin/bash")
        task.arguments = ["-c", command]

        let pipe = Pipe()
        task.standardOutput = pipe
        task.standardError = pipe

        do {
            try task.run()
        } catch {
            return nil
        }

        let data = pipe.fileHandleForReading.readDataToEndOfFile()
        task.waitUntilExit()

        return String(data: data, encoding: .utf8)
    }
}
