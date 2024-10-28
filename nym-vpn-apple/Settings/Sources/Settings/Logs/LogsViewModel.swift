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
        logs = ""
    }

    func logFileURL() -> URL? {
        logFileManager.logFileURL(logFileType: .tunnel)
    }
}

private extension LogsViewModel {
    func readLogs() {
        Task {
            guard let logFileURL = logFileManager.logFileURL(logFileType: currentLogFileType),
                  let logData = try? Data(contentsOf: logFileURL),
                  let appLogs = String(data: logData, encoding: .utf8)
            else {
                Task { @MainActor in
                    logs = ""
                }
                return
            }
            Task { @MainActor in
                logs = appLogs
            }
        }
    }
}
