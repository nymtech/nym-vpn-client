import Foundation
import Cocoa
import AppSettings
import Logging

@MainActor class AppDelegate: NSObject, NSApplicationDelegate {
    private let appSettings = AppSettings.shared

    var shouldTerminate = false

    func applicationDidFinishLaunching(_ notification: Notification) {
        NSApp.setActivationPolicy(appSettings.appMode.activationPolicy)
        NSApp.appearance = appSettings.currentAppearance.nsAppearance
    }

    func applicationDidBecomeActive(_ notification: Notification) {
        bringWindowToFront()
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

    func configureActivationPolicy(with mode: AppSetting.AppMode) {
        guard NSApp.activationPolicy() != mode.activationPolicy else { return }
        NSApp.setActivationPolicy(mode.activationPolicy)
        NSApp.activate(ignoringOtherApps: true)
        makeAppWindowVisibleAndOrderFront()
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
        NSApp.activate(ignoringOtherApps: true)
        NSApp.unhide(self)
        makeAppWindowVisibleAndOrderFront()
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

    func makeAppWindowVisibleAndOrderFront() {
        DispatchQueue.main.asyncAfter(deadline: .now()) {
            NSApp.unhide(self)
            guard let window = NSApp.windows.first,
                  window.canBecomeKey
            else {
                return
            }
            window.makeKeyAndOrderFront(self)
            window.setIsVisible(true)
        }
    }
}
