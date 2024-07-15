import SwiftUI
import Sparkle

public struct CheckForUpdatesView: View {
    @ObservedObject private var viewModel: CheckForUpdatesViewModel

    public init(viewModel: CheckForUpdatesViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        Button("Check for Updates…", action: viewModel.updater.checkForUpdates)
            .disabled(!viewModel.canCheckForUpdates)
    }
}
