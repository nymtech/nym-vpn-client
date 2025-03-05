import SwiftUI
#if os(iOS)
import ImpactGenerator
#endif
import NymLogger
import Theme

public final class LogsViewModel: ObservableObject {
    private let logFileManager: LogFileManager

#if os(iOS)
    let impactGenerator: ImpactGenerator
#endif
    let title = "logs".localizedString
    let exportLocalizedString = "logs.export".localizedString
    let deleteLocalizedString = "logs.delete".localizedString
    let noLogsLocalizedString = "logs.noLogs".localizedString
    let lineLimit = 1000

    @Published var logLines: [String] = []
    @Published var isFileExporterPresented = false
    @Published var isDeleteDialogDisplayed = false
    @Published var currentLogFileType: LogFileType = .app {
        didSet {
            readLogs()
        }
    }

    @Binding private var path: NavigationPath

    var logFileTypes: [LogFileType] {
        LogFileType.allCases
    }

    var lastLogIndex: Int {
        logLines.count - 1
    }

#if os(iOS)
    init(
        path: Binding<NavigationPath>,
        logFileManager: LogFileManager,
        impactGenerator: ImpactGenerator = ImpactGenerator.shared
    ) {
        _path = path
        self.logFileManager = logFileManager
        self.impactGenerator = impactGenerator
        readLogs()
    }
#endif
#if os(macOS)
    init(path: Binding<NavigationPath>, logFileManager: LogFileManager) {
        _path = path
        self.logFileManager = logFileManager
        readLogs()
    }
#endif

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func deleteLogs() {
        logFileManager.deleteLogs()
        logLines = []
    }

    func logFileURL() -> URL? {
        LogFileManager.logFileURL(logFileType: currentLogFileType)
    }

    func copyToPasteboard(index: Int) {
#if os(iOS)
        UIPasteboard.general.string = logLines[index]
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(logLines[index], forType: .string)
#endif
    }
}

private extension LogsViewModel {
    /// Reads the last `maxLines` lines from the log file by reading backwards in chunks.
    func readLastLinesFromFile(maxLines: Int) -> [String]? {
        guard let logFileURL = LogFileManager.logFileURL(logFileType: currentLogFileType),
              let fileHandle = try? FileHandle(forReadingFrom: logFileURL)
        else {
            return nil
        }
        defer { try? fileHandle.close() }

        let chunkSize = 4096
        let fileSize = (try? fileHandle.seekToEnd()) ?? 0
        var offset = fileSize
        var accumulatedData = Data()
        var newlineCount = 0

        // Read backwards until we have found enough newlines or reached the beginning
        while offset > 0 && newlineCount < maxLines {
            let readSizeUInt = min(UInt64(chunkSize), offset)
            let readSize = Int(readSizeUInt)
            offset -= readSizeUInt
            try? fileHandle.seek(toOffset: offset)
            guard let chunkData = try? fileHandle.read(upToCount: readSize) else {
                break
            }
            accumulatedData.insert(contentsOf: chunkData, at: 0)
            // Count newlines in the accumulated data (ASCII newline is 10).
            newlineCount = accumulatedData.reduce(0) { count, byte in
                count + (byte == 10 ? 1 : 0)
            }
        }

        guard let fullText = String(data: accumulatedData, encoding: .utf8), !fullText.isEmpty
        else {
            return nil
        }

        let allLines = fullText.components(separatedBy: "\n")
        return Array(allLines.suffix(maxLines))
    }

    func readLogs() {
        Task {
            guard let lastLines = readLastLinesFromFile(maxLines: lineLimit) else {
                await MainActor.run {
                    logLines = []
                }
                return
            }
            await MainActor.run {
                logLines = lastLines
            }
        }
    }
}
