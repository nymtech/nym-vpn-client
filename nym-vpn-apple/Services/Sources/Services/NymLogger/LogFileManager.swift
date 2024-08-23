import Foundation
import Constants

public final class LogFileManager {
    public static let shared = LogFileManager()

    private let ioQueue = DispatchQueue(label: "FileLogHandlerQueue", qos: .utility)

    private var fileHandle: FileHandle?

    init() {
        configure()
    }

    public var logFileURL: URL? {
        let fileManager = FileManager.default
        let logsDirectory = fileManager
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("net.nymtech.vpn")
            .appendingPathComponent("Logs")

        guard let logsDirectory else { return nil }

        try? fileManager.createDirectory(at: logsDirectory, withIntermediateDirectories: true, attributes: nil)
        let logFileURL = logsDirectory.appendingPathComponent(Constants.logFileName.rawValue)
        return logFileURL
    }

    public func logs() -> String {
        guard let logFileURL = logFileURL,
              let logData = try? Data(contentsOf: logFileURL),
              let appLogs = String(data: logData, encoding: .utf8)
        else {
            return ""
        }
        return appLogs
    }

    public func deleteLogs() {
        ioQueue.async {
            guard let logFileURL = self.logFileURL else { return }
            try? FileManager.default.removeItem(at: logFileURL)
            self.fileHandle = nil
        }
    }

    public func write(_ string: String) {
        ioQueue.async {
            try? self.fileHandle?.write(contentsOf: Data(string.utf8))
        }
    }
}

private extension LogFileManager {
    func configure() {
        ioQueue.async {
            guard let logFileURL = self.logFileURL else { return }

            if !FileManager.default.fileExists(atPath: logFileURL.absoluteString) {
                FileManager.default.createFile(atPath: logFileURL.relativePath, contents: nil, attributes: nil)
            }

            if self.fileHandle == nil {
                self.fileHandle = try? FileHandle(forWritingTo: logFileURL)
            }
        }
    }
}
