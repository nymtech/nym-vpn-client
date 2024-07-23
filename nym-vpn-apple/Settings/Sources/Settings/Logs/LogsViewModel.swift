import SwiftUI
import NymLogger
import Theme

public final class LogsViewModel: ObservableObject {
    let title = "logs".localizedString
    let copyLocalizedString = "copy".localizedString

    @Published var logs: String = ""

    @Binding private var path: NavigationPath

    init(path: Binding<NavigationPath>) {
        _path = path
        readLogs()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

private extension LogsViewModel {
    func readLogs() {
        guard let logFileURL = FileLogHandler.logFileURL
        else {
            logs = "logs.noLogs".localizedString
            return
        }

        if let logData = try? Data(contentsOf: logFileURL),
           let appLogs = String(data: logData, encoding: .utf8),
            !appLogs.isEmpty {
            logs = appLogs
        } else {
            logs = "logs.noLogs".localizedString
        }
    }
}
