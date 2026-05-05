import SwiftUI
import ImpactGenerator
#if os(macOS)
import GRPCManager
#endif
import NymLogger
import Theme

enum LogScrollIntent: Equatable {
    case bottom
    case preserve
    case idle
}

struct LogContent: Equatable {
    var lines: [String] = []
    var text: String = ""
    var scrollIntent: LogScrollIntent = .bottom
}

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

    private nonisolated static let initialBatchSize = 100
    private nonisolated static let lineByteCap = 2048
    private nonisolated static let defaultLineLimit = 500

    @Published private(set) var content = LogContent()
    @Published var isFileExporterPresented = false
    @Published var isShareSheetPresented = false
    @Published var isDeleteDialogDisplayed = false
    @Published var isPreparingExport = false
    @Published var exportZipURL: URL?
    @Published var currentLogFileType: LogFileType = .app {
        didSet {
            guard oldValue != currentLogFileType
            else {
                return
            }
            readLogs()
        }
    }

    @Binding private var path: NavigationPath

    private var readTask: Task<Void, Never>?
    private var prepareExportTask: Task<Void, Never>?
    private var cache: [LogFileType: [String]] = [:]
    private var lineLimitOverride: [LogFileType: Int] = [:]

    var logFileTypes: [LogFileType] {
        LogFileType.allCases
    }

    var logLines: [String] {
        content.lines
    }

    var logText: String {
        content.text
    }

    var scrollIntent: LogScrollIntent {
        content.scrollIntent
    }

    var currentLineLimit: Int {
        lineLimitOverride[currentLogFileType] ?? Self.defaultLineLimit
    }

    var hasReachedLimit: Bool {
        content.lines.count >= currentLineLimit
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
        cache.removeAll()
        lineLimitOverride.removeAll()
        exportZipURL = nil
        content = LogContent()
    }

    func loadOlder() {
        let next = currentLineLimit + Self.defaultLineLimit
        lineLimitOverride[currentLogFileType] = next
        cache[currentLogFileType] = nil
        readLogs(expanding: true)
    }

    func prepareExport() {
        prepareExportTask?.cancel()
        if let url = exportZipURL,
           FileManager.default.fileExists(atPath: url.path(percentEncoded: false)) {
            presentExport()
            return
        }
        isPreparingExport = true
        prepareExportTask = Task { [weak self] in
            let url = await Task.detached(priority: .userInitiated) {
                LogFileManager.zippedLogFilesURL()
            }.value
            if Task.isCancelled {
                return
            }
            await MainActor.run { [weak self] in
                guard let self else { return }
                self.isPreparingExport = false
                self.exportZipURL = url
                if url != nil {
                    self.presentExport()
                }
            }
        }
    }

    private func presentExport() {
#if os(iOS)
        isShareSheetPresented = true
#elseif os(macOS)
        isFileExporterPresented = true
#endif
    }
}

private extension LogsViewModel {
    func readLogs(expanding: Bool = false) {
        readTask?.cancel()
        let type = currentLogFileType
        let max = currentLineLimit
        let initialBatch = expanding ? max : min(Self.initialBatchSize, max)
        let url = LogFileManager.logFileURL(logFileType: type)

        readTask = Task { [weak self] in
            guard let self else { return }

            if !expanding, let cached = self.cache[type] {
                self.applyContent(lines: cached, expanding: false)
            }

            guard let url
            else {
                self.applyContent(lines: [], expanding: expanding)
                return
            }

            let firstChunk = await Task.detached(priority: .userInitiated) {
                Self.readLastLinesFromFile(at: url, maxLines: initialBatch, byteCap: Self.lineByteCap) ?? []
            }.value
            self.applyContent(lines: firstChunk, expanding: expanding)
            if max <= initialBatch {
                self.cache[type] = firstChunk
                return
            }

            let full = await Task.detached(priority: .background) {
                Self.readLastLinesFromFile(at: url, maxLines: max, byteCap: Self.lineByteCap) ?? []
            }.value
            self.applyContent(lines: full, expanding: expanding)
            self.cache[type] = full
        }
    }

    func applyContent(lines: [String], expanding: Bool) {
        let next = LogContent(
            lines: lines,
            text: lines.joined(separator: "\n"),
            scrollIntent: expanding ? .preserve : .bottom
        )
        guard next != content else {
            return
        }
        content = next
    }

    /// Reads the last `maxLines` lines from the file at `url` by seeking backwards in chunks.
    /// Each line is capped at `byteCap` characters to bound view layout cost on huge JSON dumps.
    nonisolated static func readLastLinesFromFile(at url: URL, maxLines: Int, byteCap: Int) -> [String]? {
        guard let fileHandle = try? FileHandle(forReadingFrom: url) else { return nil }
        defer { try? fileHandle.close() }

        let chunkSize = 4096
        let fileSize = (try? fileHandle.seekToEnd()) ?? 0
        var offset = fileSize
        var buffer = Data()
        var lineCount = 0

        while offset > 0, lineCount < maxLines {
            let readSize = Int(min(UInt64(chunkSize), offset))
            offset -= UInt64(readSize)
            try? fileHandle.seek(toOffset: offset)
            guard let chunk = try? fileHandle.read(upToCount: readSize) else { break }

            buffer.insert(contentsOf: chunk, at: 0)
            lineCount += chunk.reduce(into: 0) { $0 += ($1 == 10 /* '\n' */ ? 1 : 0) }
        }

        // If we stopped before reaching the start of the file, the leading bytes are a partial
        // line whose first byte is not guaranteed to align with a UTF-8 character boundary
        // (e.g. mid-emoji like ℹ️/⛔️). Drop everything before the first newline so decoding
        // always starts on a valid boundary; otherwise strict UTF-8 decoding fails and we
        // would wrongly report an empty file.
        if offset > 0, let firstNewline = buffer.firstIndex(of: 0x0A) {
            buffer = buffer.subdata(in: (firstNewline + 1)..<buffer.endIndex)
        }

        guard let text = String(bytes: buffer, encoding: .utf8),
              !text.isEmpty
        else {
            return nil
        }

        let lines = text.split(separator: "\n", omittingEmptySubsequences: false).map(String.init)
        let tail = Array(lines.suffix(maxLines))
        return tail.map { line in
            guard line.count > byteCap else { return line }
            return String(line.prefix(byteCap)) + "…"
        }
    }
}
