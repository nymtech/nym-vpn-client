import Foundation
import Logging
import os

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

/// A `LogHandler` that forwards messages to Apple's unified logging (`os_log`).
public final class OSLogHandler: LogHandler, @unchecked Sendable {
    private let store: OSLog
    private let lock = Lock()

    // Guarded by `lock`
    private var _metadata = Logging.Logger.Metadata()
    private var _logLevel: Logging.Logger.Level = .info

    // MARK: - Init

    public init(subsystem: String, category: String) {
        self.store = OSLog(subsystem: subsystem, category: category)
    }

    // MARK: - LogHandler conformance (use fully-qualified Logging.Logger.*)

    public var metadata: Logging.Logger.Metadata {
        get { lock.with { _metadata } }
        set { lock.with { _metadata = newValue } }
    }

    public var logLevel: Logging.Logger.Level {
        get { lock.with { _logLevel } }
        set { lock.with { _logLevel = newValue } }
    }

    public subscript(metadataKey key: String) -> Logging.Logger.Metadata.Value? {
        get { lock.with { _metadata[key] } }
        set { lock.with { _metadata[key] = newValue } }
    }

    // swiftlint:disable:next function_parameter_count
    public func log(
        level: Logging.Logger.Level,
        message: Logging.Logger.Message,
        metadata: Logging.Logger.Metadata?,
        source: String,
        file: String,
        function: String,
        line: UInt
    ) {
        // Snapshot handler metadata
        let base = lock.with { _metadata }

        // Merge per-call metadata
        var full = base
        if let metadata { full.merge(metadata) { $1 } }

        let metadataOutput = formatMetadata(full)
        let sourceInfo = "[\(source)] "
        let logLine = "\(sourceInfo)\(metadataOutput)\(message)\n"

        os_log("%{public}s", log: store, type: level.osLogType, logLine)
    }
}

// MARK: - Helpers (no global extensions to avoid redeclaration)

private func formatMetadata(_ md: Logging.Logger.Metadata) -> String {
    guard !md.isEmpty else { return "" }
    return md.map { "\($0.key)=\($0.value)" }.joined(separator: " ") + " "
}

private extension Logging.Logger.Level {
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
