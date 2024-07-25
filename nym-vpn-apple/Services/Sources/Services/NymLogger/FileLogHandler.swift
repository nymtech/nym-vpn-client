import Foundation
import Logging
import Constants

public class FileLogHandler: LogHandler {
    private let label: String

    public init(label: String) {
        self.label = label
    }

    public var metadata = Logging.Logger.Metadata()
    public var logLevel = Logging.Logger.Level.info

    private var fileHandle: FileHandle?
    private var fileLock = NSLock()

    public static var logFileURL: URL? {
        let fileManager = FileManager.default
        let logsDirectory = fileManager
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("net.nymtech.vpn")
            .appendingPathComponent("Logs")

        guard let logsDirectory else { return nil }

        try? fileManager.createDirectory(at: logsDirectory, withIntermediateDirectories: true, attributes: nil)
        let logFileURL = logsDirectory.appendingPathComponent("Log.log")
        return logFileURL
    }

    public static func deleteLogs() {
        guard let logFileURL = FileLogHandler.logFileURL else { return }
        try? FileManager.default.removeItem(at: logFileURL)
    }

    public subscript(metadataKey key: String) -> Logging.Logger.Metadata.Value? {
        get { metadata[key] }
        set { metadata[key] = newValue }
    }

    public func log(
        level: Logging.Logger.Level,
        message: Logging.Logger.Message,
        metadata: Logging.Logger.Metadata?,
        file: String,
        function: String,
        line: UInt
    ) {
        var fullMetadata = self.metadata
        if let metadata = metadata {
            fullMetadata.merge(metadata) { $1 }
        }

        var metadataOutput = fullMetadata.formatted()
        if !metadataOutput.isEmpty {
            metadataOutput = " " + metadataOutput
        }

        let logLine = "\(Date()) [\(label)] \(level.emoji) \(level)\(metadataOutput): \(message)\n"
        let data = Data(logLine.utf8)

        let fileHandle = fileLock.withLock { () -> FileHandle? in
            if let fileHandle = self.fileHandle {
                return fileHandle
            } else if let logFileURL = FileLogHandler.logFileURL {
                self.fileHandle = try? FileHandle(forWritingTo: logFileURL)
                return self.fileHandle
            } else {
                return nil
            }
        }

        try? fileHandle?.write(contentsOf: data)
    }
}

extension Logging.Logger.Metadata {
    func formatted() -> String {
        map { key, value in "\(key)=\(value)" }
            .joined(separator: " ")
    }
}

extension Logging.Logger.Level {
    var emoji: String {
        switch self {
        case .trace:
            return "👀"
        case .debug:
            return "⌨️"
        case .info:
            return "ℹ"
        case .notice:
            return "📣"
        case .warning:
            return "⚠️"
        case .error:
            return "⛔️"
        case .critical:
            return "🔥"
        }
    }
}
