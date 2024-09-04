import SwiftUI
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
            guard let logFileURL = self.logFileURL(logFileType: self.logFileType) else { return }
            if let appLogFileURL = self.logFileURL(logFileType: .app) {
                try? FileManager.default.removeItem(at: appLogFileURL)
            }
            if let tunnelLogFileURL = self.logFileURL(logFileType: .tunnel) {
                try? FileManager.default.removeItem(at: tunnelLogFileURL)
            }
            try? self.fileHandle?.close()
            try? FileManager.default.removeItem(at: logFileURL)
            self.fileHandle = nil

            DarwinNotificationCenter.shared.post(name: DarwinNotificationKey.reconfigureLogs.rawValue)
        }
    }
}

private extension LogFileManager {
    func setup() {
        notificationObservation = DarwinNotificationCenter.shared.addObserver(
            name: DarwinNotificationKey.reconfigureLogs.rawValue
        ) { [weak self] in
            self?.fileHandle = nil
            self?.configure()
        }
    }

    func configure() {
        ioQueue.async {
            guard let logFileURL = self.logFileURL(logFileType: self.logFileType) else { return }

            if !FileManager.default.fileExists(atPath: logFileURL.relativePath) {
                FileManager.default.createFile(atPath: logFileURL.relativePath, contents: nil, attributes: nil)
            }

            if self.fileHandle == nil {
                self.fileHandle = try? FileHandle(forWritingTo: logFileURL)
                _ = try? self.fileHandle?.seekToEnd()
            }
        }
    }
}
