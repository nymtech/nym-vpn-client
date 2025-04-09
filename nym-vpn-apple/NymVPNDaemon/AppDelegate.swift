import Foundation
import Cocoa
import AppSettings

class AppDelegate: NSObject, NSApplicationDelegate {
    private let appSettings = AppSettings.shared

    var shouldTerminate = false
    var terminationType: TerminationType?

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        quit(sender)
    }
}

private extension AppDelegate {
    func quit(_ app: NSApplication) -> NSApplication.TerminateReply {
        // Dock icon
        if terminationType == nil {
            return .terminateNow
        }

        // App or menubar
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

enum TerminationType {
    case app
    case menubar
}
