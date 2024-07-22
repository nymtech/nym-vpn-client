import SwiftUI
import Sparkle

public final class CheckForUpdatesViewModel: ObservableObject {
    public var updater: SPUUpdater

    @Published public var canCheckForUpdates = false

    public init(updater: SPUUpdater) {
        self.updater = updater
        updater.publisher(for: \.canCheckForUpdates)
            .assign(to: &$canCheckForUpdates)
    }
}
