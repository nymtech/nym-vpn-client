import SwiftUI
import Sparkle

public final class AutoUpdater {
    public static let shared = AutoUpdater()
    private let updaterController: SPUStandardUpdaterController

    public var updater: SPUUpdater {
        updaterController.updater
    }

    public init() {
        // If you want to start the updater manually, pass false to startingUpdater and call .startUpdater() later
        // This is where you can also pass an updater delegate if you need one
        updaterController = SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: nil,
            userDriverDelegate: nil
        )
    }
}
