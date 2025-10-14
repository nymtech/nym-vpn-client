import Foundation
import Logging

public final class NymLogger {
    public let logFileManager: LogFileManager

    public init() {
        let manager = LogFileManager(logFileType: .app)
        self.logFileManager = manager

        Bootstrap.once(using: manager)
    }
}

private enum Bootstrap {
    private static var didBootstrap = false
    private static var lock = DispatchQueue(label: "Bootstrap.Lock")

    static func once(using manager: LogFileManager) {
        lock.sync {
            guard !didBootstrap else { return }
            LoggingSystem.bootstrap { label in
                FileLogHandler(label: label, logFileManager: manager)
            }
            didBootstrap = true
        }
    }
}
