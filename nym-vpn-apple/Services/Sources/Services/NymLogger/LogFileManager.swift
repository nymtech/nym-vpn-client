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

    public static func logFileURL(logFileType: LogFileType) -> URL? {
        let fileManager = FileManager.default
        var logsDirectory: URL?
#if os(macOS)
        switch logFileType {
        case .app:
            logsDirectory = try? fileManager
                .url(for: .libraryDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
        case .daemon:
            return URL(fileURLWithPath: "/var/log/nym-vpnd/nym-vpnd.log")
        }

#else
        logsDirectory = fileManager
            .containerURL(
                forSecurityApplicationGroupIdentifier: Constants.groupID.rawValue
            )
#endif

        guard var logsDirectory else { return nil }
        logsDirectory = logsDirectory
            .appendingPathComponent("net.nymtech.vpn")
            .appendingPathComponent("Logs")

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
                guard let logFileURL = LogFileManager.logFileURL(logFileType: type) else { return }
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

        guard let logFileURL = LogFileManager.logFileURL(logFileType: self.logFileType) else { return }

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
