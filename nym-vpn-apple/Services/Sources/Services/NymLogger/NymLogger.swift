import Logging

public final class NymLogger {
    private let label: String

    public let logger: Logger

    public init(label: String) {
        self.label = label
        self.logger = Logger(label: label) { label in
            FileLogHandler(label: label)
        }
    }
}
