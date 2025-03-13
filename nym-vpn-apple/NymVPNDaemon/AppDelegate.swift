import Foundation
import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
    var shouldTerminate = false

    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        quit(sender)
    }

    private func quit(_ app: NSApplication) -> NSApplication.TerminateReply {
//        let keepMenuBarItemRunningOnQuit = Global.groupDefaults.bool(forKey: "keepMenuBarRunningOnQuit")
        let keepMenuBarItemRunningOnQuit = true
        guard !shouldTerminate, keepMenuBarItemRunningOnQuit, app.activationPolicy() != .accessory else { return .terminateNow }

        // Item-0 - name of menu bar extra item
        app.windows.filter { $0.title != "Item-0" }.forEach { $0.close() }
        app.setActivationPolicy(.accessory)
        return .terminateCancel
    }
}
