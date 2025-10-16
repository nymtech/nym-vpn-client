import SwiftUI
import ImpactGenerator
#if os(macOS)
import GRPCManager
#endif
import NymLogger
import Theme

@MainActor public final class LogsViewModel: ObservableObject {
    private let logFileManager: LogFileManager

    let impactGenerator: ImpactGenerator
#if os(macOS)
    let grpcManager: GRPCManager
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
        impactGenerator: ImpactGenerator
    ) {
        _path = path
        self.logFileManager = logFileManager
        self.impactGenerator = impactGenerator
        readLogs()
    }
#elseif os(macOS)
    init(
        path: Binding<NavigationPath>,
        logFileManager: LogFileManager,
        impactGenerator: ImpactGenerator,
        grpcManager: GRPCManager
    ) {
        _path = path
        self.logFileManager = logFileManager
            self.impactGenerator = impactGenerator
        self.grpcManager = grpcManager
        readLogs()
    }
#endif

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func deleteLogs() {
#if os(macOS)
        Task {
            try? await grpcManager.deleteLog()
        }
#endif
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
    /// Reads the last `maxLines` lines from the file at `url` by seeking backwards in chunks.
    nonisolated static func readLastLinesFromFile(at url: URL, maxLines: Int) -> [String]? {
        guard let fileHandle = try? FileHandle(forReadingFrom: url) else { return nil }
        defer { try? fileHandle.close() }

        let chunkSize = 4096
        let fileSize = (try? fileHandle.seekToEnd()) ?? 0
        var offset = fileSize
        var buffer = Data()
        var lineCount = 0

        // Read backwards until enough newlines or start of file
        while offset > 0, lineCount < maxLines {
            let readSize = Int(min(UInt64(chunkSize), offset))
            offset -= UInt64(readSize)
            try? fileHandle.seek(toOffset: offset)
            guard let chunk = try? fileHandle.read(upToCount: readSize) else { break }

            // Prepend the chunk
            buffer.insert(contentsOf: chunk, at: 0)

            // Count only the new chunk’s newlines to avoid O(n^2)
            lineCount += chunk.reduce(into: 0) { $0 += ($1 == 10 /* '\n' */ ? 1 : 0) }
        }

        guard let text = String(data: buffer, encoding: .utf8), !text.isEmpty else { return nil }
        let lines = text.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
        return Array(lines.suffix(maxLines))
    }

    func readLogs() {
        let url = LogFileManager.logFileURL(logFileType: currentLogFileType)
        let max = lineLimit

        Task.detached(priority: .utility) { [weak self] in
            guard let url else {
                await MainActor.run { [weak self] in
                    self?.logLines = []
                }
                return
            }
            let lastLines = Self.readLastLinesFromFile(at: url, maxLines: max) ?? []
            await MainActor.run { [weak self] in
                self?.logLines = lastLines
            }
        }
    }
}
