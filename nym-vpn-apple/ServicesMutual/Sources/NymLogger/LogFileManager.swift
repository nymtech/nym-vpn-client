import Foundation
import Combine
import Constants
import DarwinNotificationCenter

public final class LogFileManager: ObservableObject {
    private let ioQueue = DispatchQueue(label: "LogFileManagerQueue", qos: .utility)
    private let logFileType: LogFileType

    private var fileHandle: FileHandle?
    private var notificationObservation: Cancellable?

    private let maxFileSize: UInt64 = 5 * 1024 * 1024  // 5 MB
    private let maxFileAge: TimeInterval = 7 * 24 * 60 * 60  // 1 week

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
                .url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
        case .daemon:
            return URL(fileURLWithPath: "/var/log/nym-vpnd/nym-vpnd.log")
        }
#elseif os(iOS)
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
        return logsDirectory.appendingPathComponent(fileName)
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
        deleteIfNeeded(at: logFileURL)

        let fileManager = FileManager.default
        if !fileManager.fileExists(atPath: logFileURL.path(percentEncoded: false)) {
            fileManager.createFile(
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

    /// Delete the log file if it exceeds size or age thresholds
    func deleteIfNeeded(at url: URL) {
        let fileManager = FileManager.default
        do {
            let attrs = try fileManager.attributesOfItem(atPath: url.path(percentEncoded: false))
            let fileSize = attrs[.size] as? UInt64 ?? 0
            let modDate = attrs[.modificationDate] as? Date ?? Date.distantPast
            let age = Date().timeIntervalSince(modDate)

            if fileSize >= maxFileSize || age >= maxFileAge {
                try fileManager.removeItem(at: url)
            }
        } catch {
            print("Log deletion error: \(error)")
        }
    }
}
