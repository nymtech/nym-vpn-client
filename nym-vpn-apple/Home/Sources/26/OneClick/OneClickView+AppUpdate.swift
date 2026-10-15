import SwiftUI
import ConfigurationManager
import Constants
import ExternalLinkManager

extension OneClickView {
    var appUpdatePresented: Binding<Bool> {
        Binding(
            get: {
                _ = appUpdateGeneration
                return configuration.showsAppUpdatePrompt &&
                    (configuration.blocksConnectForAppUpdate || !configuration.appUpdatePromptDismissed)
            },
            set: { presented in
                guard !presented else { return }
                if configuration.blocksConnectForAppUpdate {
                    appUpdateGeneration += 1
                } else {
                    configuration.appUpdatePromptDismissed = true
                }
            }
        )
    }

    func openUpdatePage() {
        try? ExternalLinkManager.shared.openExternalURL(urlString: Constants.downloadLink.rawValue)
    }

    var connectButtonDisabled: Bool {
        if configuration.blocksConnectForAppUpdate &&
            viewModel.connectState != .connected &&
            viewModel.connectState != .stop {
            return true
        }
        switch viewModel.connectState {
        case .connecting, .disconnecting, .noInternet:
            true
        case .disconnected, .stop, .connected, .noSubscription:
            false
        }
    }
}
