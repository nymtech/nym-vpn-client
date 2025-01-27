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
    func readLogs() {
        Task {
            guard let logFileURL = LogFileManager.logFileURL(logFileType: currentLogFileType),
                  let logData = try? Data(contentsOf: logFileURL),
                  let appLogs = String(data: logData, encoding: .utf8)
            else {
                await MainActor.run {
                    logLines = []
                }
                return
            }
            let logLinesArray = appLogs.split(separator: "\n").map { String($0) }
            await MainActor.run {
                logLines.replaceSubrange(0..<logLines.count, with: logLinesArray)
            }
        }
    }
}
