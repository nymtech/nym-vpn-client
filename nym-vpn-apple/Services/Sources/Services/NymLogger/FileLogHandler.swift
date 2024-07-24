import Foundation
import Logging

public class FileLogHandler: LogHandler {
    private let fileHandle: FileHandle
    private let label: String

    public init(label: String) {
        self.label = label

        do {
            guard let logFileURL = FileLogHandler.logFileURL
            else {
                fatalError("Cannot create FileLogHandler")
            }
            self.fileHandle = try FileHandle(forWritingTo: logFileURL)
            try self.fileHandle.seekToEnd()
        } catch let error {
            fatalError("Cannot create FileLogHandler: \(error.localizedDescription)")
        }
    }

    public var metadata = Logger.Metadata()
    public var logLevel = Logger.Level.info

    public static var logFileURL: URL? {
        let fileManager = FileManager.default
        let logsDirectory = try? fileManager.url(
            for: .applicationSupportDirectory,
            in: .userDomainMask,
            appropriateFor: nil,
            create: true
        )
        .appendingPathComponent("net.nymtech.vpn")
        .appendingPathComponent("Logs")

        guard let logsDirectory else { return nil }

        try? fileManager.createDirectory(at: logsDirectory, withIntermediateDirectories: true, attributes: nil)
        let logFileURL = logsDirectory.appendingPathComponent("Log.log")
        if !fileManager.fileExists(atPath: logFileURL.path()) {
            try? Data().write(to: logFileURL)
        }
        return logFileURL
    }

    public subscript(metadataKey key: String) -> Logger.Metadata.Value? {
        get { metadata[key] }
        set { metadata[key] = newValue }
    }

    public func log(
        level: Logger.Level,
        message: Logger.Message,
        metadata: Logger.Metadata?,
        file: String,
        function: String,
        line: UInt
    ) {
        var fullMetadata = self.metadata
        if let metadata = metadata {
            fullMetadata.merge(metadata) { $1 }
        }
        let logLine = "\(Date()) [\(self.label)] \(level): \(message)\n"
        if let data = logLine.data(using: .utf8) {
            fileHandle.write(data)
        }
    }
}
