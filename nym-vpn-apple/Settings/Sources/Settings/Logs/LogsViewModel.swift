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

    @Binding private var path: NavigationPath

    init(path: Binding<NavigationPath>, logFileManager: LogFileManager = LogFileManager.shared) {
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
        logFileManager.logFileURL
    }
}

private extension LogsViewModel {
    func readLogs() {
        logs = logFileManager.logs()
    }
}
