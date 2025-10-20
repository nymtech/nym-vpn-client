import Foundation
import Logging
import Constants

private final class Lock {
    private let lock = NSLock()

    @discardableResult func with<T>(_ body: () throws -> T) rethrows -> T {
        lock.lock()
        defer {
            lock.unlock()
        }
        return try body()
    }
}

public final class FileLogHandler: LogHandler, @unchecked Sendable {
    private let label: String
    private let logFileManager: LogFileManager
    private let lock = Lock()

    // Backing storage guarded by lock
    private var _metadata = Logger.Metadata()
    private var _logLevel: Logger.Level = .info

    public init(label: String, logFileManager: LogFileManager) {
        self.label = label
        self.logFileManager = logFileManager
    }

    // MARK: LogHandler requirements

    public var metadata: Logger.Metadata {
        get { lock.with { _metadata } }
        set { lock.with { _metadata = newValue } }
    }

    public var logLevel: Logger.Level {
        get { lock.with { _logLevel } }
        set { lock.with { _logLevel = newValue } }
    }

    public subscript(metadataKey key: String) -> Logger.Metadata.Value? {
        get { lock.with { _metadata[key] } }
        set { lock.with { _metadata[key] = newValue } }
    }

    // swiftlint:disable:next function_parameter_count
    public func log(
        level: Logger.Level,
        message: Logger.Message,
        metadata: Logger.Metadata?,
        source: String,
        file: String,
        function: String,
        line: UInt
    ) {
        // snapshot current metadata thread-safely
        let baseMeta = lock.with { _metadata }
        var fullMetadata = baseMeta
        if let metadata { fullMetadata.merge(metadata) { $1 } }

        var metadataOutput = fullMetadata.formatted()
        if !metadataOutput.isEmpty { metadataOutput = " " + metadataOutput }

        let logLine = "\(Date()) [\(label)] \(level.emoji) \(level)\(metadataOutput): \(message)\n"

        // LogFileManager.write is @MainActor → hop safely
        Task { @MainActor [logFileManager, logLine] in
            logFileManager.write(logLine)
        }
    }
}

// MARK: - Helpers

extension Logging.Logger.Metadata {
    func formatted() -> String {
        map { key, value in "\(key)=\(value)" }.joined(separator: " ")
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
