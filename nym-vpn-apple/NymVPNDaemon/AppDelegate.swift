import Foundation
import Cocoa
import AppSettings

class AppDelegate: NSObject, NSApplicationDelegate {
    private let appSettings = AppSettings.shared

    var shouldTerminate = false

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        quit(sender)
    }
}

private extension AppDelegate {
    func quit(_ app: NSApplication) -> NSApplication.TerminateReply {
        guard !shouldTerminate, shouldKeepMenuBarItemRunningOnQuit()
        else {
            return .terminateNow
        }

        return .terminateCancel
    }

    func shouldKeepMenuBarItemRunningOnQuit() -> Bool {
        switch appSettings.appMode {
        case .both, .menubarOnly:
            true
        case .dockOnly:
            false
        }
    }
}
