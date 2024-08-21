import Foundation
import Logging
import os

public class OSLogHandler: LogHandler {
    private let label: String
    private let store: OSLog

    public var metadata = Logging.Logger.Metadata()
    public var logLevel = Logging.Logger.Level.info

    public init(subsystem: String, category: String) {
        self.label = category
        self.store = OSLog(subsystem: subsystem, category: label)
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
            metadataOutput += " "
        }
        let logLine = "\(metadataOutput)\(message)\n"

        os_log("%{public}s", log: store, type: level.osLogType, "\(logLine)")
    }
}

extension Logging.Logger.Level {
    var osLogType: OSLogType {
        switch self {
        case .trace, .debug:
            return .debug
        case .info, .notice, .warning:
            return .info
        case .error, .critical:
            return .error
        }
    }
}
