import Foundation
import Cocoa
import AppSettings

class AppDelegate: NSObject, NSApplicationDelegate {
    private let appSettings = AppSettings.shared

    // set by your SwiftUI “quitApp(from:)” before calling .terminate()
    var shouldTerminate = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        configureActivationPolicy(appSettings.appMode)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        switch appSettings.appMode {
        case .dockOnly:
            true
        case .both, .menubarOnly:
            false
        }
    }

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        shouldTerminate ? .terminateNow : .terminateCancel
    }

    func configureActivationPolicy(_ mode: AppSetting.AppMode) {
        switch mode {
        case .menubarOnly:
            NSApp.setActivationPolicy(.accessory)
        case .dockOnly, .both:
            NSApp.setActivationPolicy(.regular)
        }
    }
}

private extension AppDelegate {
    func quit(_ app: NSApplication) -> NSApplication.TerminateReply {
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
