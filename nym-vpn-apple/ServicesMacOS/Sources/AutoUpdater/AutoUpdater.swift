import SwiftUI
import Sparkle

@MainActor public final class AutoUpdater: NSObject {
    public static let shared = AutoUpdater()

    public var didPrepareForQuit = false

    private lazy var updaterController: SPUStandardUpdaterController = {
        SPUStandardUpdaterController(
            startingUpdater: true,
            updaterDelegate: self,
            userDriverDelegate: nil
        )
    }()

    public var updater: SPUUpdater { updaterController.updater }

    public init(didPrepareForQuit: Bool = false) {
        self.didPrepareForQuit = didPrepareForQuit
    }
}

extension AutoUpdater: SPUUpdaterDelegate {
    nonisolated public func updaterWillRelaunchApplication(_ updater: SPUUpdater) {
        Task { @MainActor [weak self] in
            guard let self else { return }
            self.didPrepareForQuit = true
            NSApp.setActivationPolicy(.regular)
        }
    }
}
