import SwiftUI
import Sparkle

public final class AutoUpdater: NSObject {
    public static let shared = AutoUpdater()

    public var didPrepareForQuit = false

    private lazy var updaterController: SPUStandardUpdaterController = {
        SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: self,
            userDriverDelegate: nil
        )
    }()

    public var updater: SPUUpdater {
        updaterController.updater
    }
}

extension AutoUpdater: SPUUpdaterDelegate {
    public func updaterWillRelaunchApplication(_ updater: SPUUpdater) {
        didPrepareForQuit = true
        NSApp.setActivationPolicy(.regular)
    }
}
