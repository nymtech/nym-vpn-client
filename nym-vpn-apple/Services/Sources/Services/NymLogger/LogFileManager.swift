import Foundation
import Combine
import Constants
import DarwinNotificationCenter

public final class LogFileManager: ObservableObject {
    private let ioQueue = DispatchQueue(label: "LogFileManagerQueue", qos: .utility)
    private let logFileType: LogFileType

    private var fileHandle: FileHandle?
    private var notificationObservation: Cancellable?

    public init(logFileType: LogFileType) {
        self.logFileType = logFileType

        setup()
        configure()
    }

    deinit {
        try? fileHandle?.close()
        fileHandle = nil
    }

    public func logFileURL(logFileType: LogFileType) -> URL? {
        let fileManager = FileManager.default
        let logsDirectory = fileManager
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )?
            .appendingPathComponent("net.nymtech.vpn")
            .appendingPathComponent("Logs")

        guard let logsDirectory else { return nil }

        try? fileManager.createDirectory(at: logsDirectory, withIntermediateDirectories: true, attributes: nil)
        let fileName = "\(logFileType.rawValue)\(Constants.logFileName.rawValue)"
        let logFileURL = logsDirectory.appendingPathComponent(fileName)

        return logFileURL
    }

    public func write(_ string: String) {
        ioQueue.async {
            try? self.fileHandle?.write(contentsOf: Data(string.utf8))
        }
    }

    public func deleteLogs() {
        ioQueue.async {
            LogFileType.allCases.forEach { type in
                guard let logFileURL = self.logFileURL(logFileType: type) else { return }
                try? FileManager.default.removeItem(at: logFileURL)
            }
            try? self.fileHandle?.close()
            self.fileHandle = nil

            DarwinNotificationCenter.shared.post(name: DarwinNotificationKey.reconfigureLogs.key)
        }
    }
}

private extension LogFileManager {
    func setup() {
        notificationObservation = DarwinNotificationCenter.shared.addObserver(
            name: DarwinNotificationKey.reconfigureLogs.key
        ) { [weak self] in
            self?.ioQueue.async {
                self?.fileHandle = nil
                self?.configureNoQueue()
            }
        }
    }

    func configure() {
        ioQueue.async {
            self.configureNoQueue()
        }
    }

    func configureNoQueue() {
        dispatchPrecondition(condition: .onQueue(ioQueue))

        guard let logFileURL = self.logFileURL(logFileType: self.logFileType) else { return }

        if !FileManager.default.fileExists(atPath: logFileURL.path(percentEncoded: false)) {
            FileManager.default.createFile(
                atPath: logFileURL.path(percentEncoded: false),
                contents: nil,
                attributes: nil
            )
        }

        if self.fileHandle == nil {
            self.fileHandle = try? FileHandle(forWritingTo: logFileURL)
            _ = try? self.fileHandle?.seekToEnd()
        }
    }
}
