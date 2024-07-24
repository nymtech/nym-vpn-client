import Foundation
import Logging
import Constants

public class FileLogHandler: LogHandler {
    private let label: String

    public init(label: String) {
        self.label = label
    }

    public var metadata = Logger.Metadata()
    public var logLevel = Logger.Level.info

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

        let symbol: String
        switch level {
        case .trace:
            symbol = "👀"
        case .debug:
            symbol = "⌨️"
        case .info:
            symbol = "ℹ"
        case .notice:
            symbol = "📣"
        case .warning:
            symbol = "⚠️"
        case .error:
            symbol = "⛔️"
        case .critical:
            symbol = "🔥"
        }

        let logLine = "\(Date()) [\(label)] \(symbol) \(level): \(message)\n"

        guard let data = logLine.data(using: .utf8),
              let logFileURL = FileLogHandler.logFileURL
        else {
            return
        }

        if FileManager.default.fileExists(atPath: logFileURL.path()) {
            let fileHandle = try? FileHandle(forWritingTo: logFileURL)
            fileHandle?.seekToEndOfFile()
            fileHandle?.write(data)
            fileHandle?.closeFile()
        } else {
            try? data.write(to: logFileURL, options: .atomic)
        }
    }
}
