import Foundation
import Logging
import os

public class OSLogHandler: LogHandler {
    private let store: OSLog

    public var metadata = Logger.Metadata()
    public var logLevel = Logger.Level.info

    public init(subsystem: String, category: String) {
        self.store = OSLog(subsystem: subsystem, category: category)
    }

    public subscript(metadataKey key: String) -> Logging.Logger.Metadata.Value? {
        get { metadata[key] }
        set { metadata[key] = newValue }
    }

    // MARK: - Required LogHandler implementation

    /// This is the new required method (with `source:`) that replaces the deprecated default impl.
    public func log(
        level: Logging.Logger.Level,
        message: Logging.Logger.Message,
        metadata: Logging.Logger.Metadata?,
        source: String,
        file: String,
        function: String,
        line: UInt
    ) {
        // Merge in any per-handler metadata
        var fullMetadata = self.metadata
        if let metadata = metadata {
            fullMetadata.merge(metadata) { $1 }
        }

        // Format metadata as key=value pairs
        var metadataOutput = fullMetadata.formatted()
        if !metadataOutput.isEmpty {
            metadataOutput += " "
        }

        // You can choose whether to include `source` in your output.
        // Here we prepend it in square brackets, but you could omit it if you prefer.
        let sourceInfo = "[\(source)] "
        let logLine = "\(sourceInfo)\(metadataOutput)\(message)\n"

        os_log("%{public}s", log: store, type: level.osLogType, logLine)
    }
}

extension Logging.Logger.Level {
    fileprivate var osLogType: OSLogType {
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
