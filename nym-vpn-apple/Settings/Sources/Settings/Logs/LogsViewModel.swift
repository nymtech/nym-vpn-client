import SwiftUI
import NymLogger
import Theme

public final class LogsViewModel: ObservableObject {
    let title = "logs".localizedString
    let exportLocalizedString = "logs.export".localizedString
    let deleteLocalizedString = "logs.delete".localizedString
    let noLogsLocalizedString = "logs.noLogs".localizedString

    @Published var logs: String = ""
    @Published var isFileExporterPresented = false
    @Published var isDeleteDialogDisplayed = false

    @Binding private var path: NavigationPath

    init(path: Binding<NavigationPath>) {
        _path = path
        readLogs()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func deleteLogs() {
        FileLogHandler.deleteLogs()
        readLogs()
    }

    func logFileURL() -> URL? {
        FileLogHandler.logFileURL
    }
}

private extension LogsViewModel {
    func readLogs() {
        guard let logFileURL = FileLogHandler.logFileURL,
              let logData = try? Data(contentsOf: logFileURL),
              let appLogs = String(data: logData, encoding: .utf8)
        else {
            logs = ""
            return
        }
        logs = appLogs
    }
}
