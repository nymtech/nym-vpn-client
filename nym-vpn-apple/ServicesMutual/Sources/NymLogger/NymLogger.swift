import Logging

@MainActor public class NymLogger {
    public let logFileManager: LogFileManager

    public init() {
        let newLogFileManager = LogFileManager(logFileType: .app)
        self.logFileManager = newLogFileManager
        LoggingSystem.bootstrap { label in
            FileLogHandler(label: label, logFileManager: newLogFileManager)
        }
    }
}
