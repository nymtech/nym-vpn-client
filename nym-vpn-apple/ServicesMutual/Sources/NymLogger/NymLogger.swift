import Foundation
import Logging

public final class NymLogger {
    public let logFileManager: LogFileManager

    public init() {
        logFileManager = Self.sharedLogFileManager
    }

    private static let sharedLogFileManager: LogFileManager = {
        let manager = LogFileManager(logFileType: .app)

        LoggingSystem.bootstrap { label in
            let fileLogger = FileLogHandler(label: label, logFileManager: manager)

#if DEBUG
            return MultiplexLogHandler([
                StreamLogHandler.standardOutput(label: label),
                fileLogger
            ])
#else
            return fileLogger
#endif
        }

        return manager
    }()
}
