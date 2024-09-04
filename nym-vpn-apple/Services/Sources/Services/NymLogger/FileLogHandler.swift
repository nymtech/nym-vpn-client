import Foundation
import Logging
import Constants

public class FileLogHandler: LogHandler {
    private let label: String
    private let logFileManager: LogFileManager

    public init(label: String, logFileManager: LogFileManager) {
        self.label = label
        self.logFileManager = logFileManager
    }

    public var metadata = Logging.Logger.Metadata()
    public var logLevel = Logging.Logger.Level.info

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

        logFileManager.write(logLine)
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
