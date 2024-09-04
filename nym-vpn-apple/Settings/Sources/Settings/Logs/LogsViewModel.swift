import SwiftUI
import NymLogger
import Theme

public final class LogsViewModel: ObservableObject {
    private let logFileManager: LogFileManager

    let title = "logs".localizedString
    let exportLocalizedString = "logs.export".localizedString
    let deleteLocalizedString = "logs.delete".localizedString
    let noLogsLocalizedString = "logs.noLogs".localizedString

    @Published var logs: String = ""
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

    init(path: Binding<NavigationPath>, logFileManager: LogFileManager) {
        _path = path
        self.logFileManager = logFileManager
        readLogs()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func deleteLogs() {
        logFileManager.deleteLogs()
        logs = ""
    }

    func logFileURL() -> URL? {
        logFileManager.logFileURL(logFileType: .tunnel)
    }
}

private extension LogsViewModel {
    func readLogs() {
        guard let logFileURL = logFileManager.logFileURL(logFileType: currentLogFileType),
              let logData = try? Data(contentsOf: logFileURL),
              let appLogs = String(data: logData, encoding: .utf8)
        else {
            logs = ""
            return
        }
        logs = appLogs
    }
}
