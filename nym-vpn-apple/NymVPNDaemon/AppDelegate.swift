import Foundation
import Cocoa
import AppSettings
import Logging

class AppDelegate: NSObject, NSApplicationDelegate {
    private let appSettings = AppSettings.shared

    var shouldTerminate = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        configureActivationPolicy(appSettings.appMode)
        NSApplication.shared.delegate = self
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

    func applicationWillFinishLaunching(_ notification: Notification) {
        NSAppleEventManager.shared()
            .setEventHandler(
                self,
                andSelector: #selector(handleQuitEvent(_:withReplyEvent:)),
                forEventClass: AEEventClass(kCoreEventClass),
                andEventID: AEEventID(kAEQuitApplication)
            )
    }

    func applicationShouldHandleReopen(_ sender: NSApplication, hasVisibleWindows: Bool) -> Bool {
        if !hasVisibleWindows {
            bringWindowToFront()
        }
        return true
    }
}

extension AppDelegate {
    func bringWindowToFront() {
        NSApp.windows.first?.makeKeyAndOrderFront(self)

        if #available(macOS 14.0, *) {
            NSApplication.shared.activate()
        } else {
            NSApp.activate(ignoringOtherApps: true)
        }
    }
}

private extension AppDelegate {
    @objc func handleQuitEvent(
        _ event: NSAppleEventDescriptor,
        withReplyEvent replyEvent: NSAppleEventDescriptor
    ) {
        shouldTerminate = true
        NSApplication.shared.terminate(self)
    }

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
