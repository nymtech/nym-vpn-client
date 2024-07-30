import SwiftUI
import NymLogger
import Theme

public final class LogsViewModel: ObservableObject {
#if os(iOS)
    let pasteboard: UIPasteboard
#endif
#if os(macOS)
    let pasteboard: NSPasteboard
#endif

    let title = "logs".localizedString
    let copyLocalizedString = "copy".localizedString
    let deleteLocalizedString = "logs.delete".localizedString

    @Published var logs: String = ""

    @Binding private var path: NavigationPath
#if os(iOS)
    init(path: Binding<NavigationPath>, pasteboard: UIPasteboard = UIPasteboard.general) {
        _path = path
        self.pasteboard = pasteboard
        readLogs()
    }
#endif
#if os(macOS)
    init(path: Binding<NavigationPath>, pasteboard: NSPasteboard = NSPasteboard.general) {
        _path = path
        self.pasteboard = pasteboard
        pasteboard.declareTypes([.string], owner: nil)
        readLogs()
    }
#endif

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func copyToPasteBoard() {
#if os(macOS)
        pasteboard.setString(logs, forType: .string)
#endif
#if os(iOS)
        pasteboard.string = logs
#endif
    }

    func deleteLogs() {
        FileLogHandler.deleteLogs()
        readLogs()
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
