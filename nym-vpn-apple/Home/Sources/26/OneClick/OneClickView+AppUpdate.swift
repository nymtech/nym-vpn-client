import SwiftUI
import ConfigurationManager
import Constants
import ExternalLinkManager

extension OneClickView {
    var appUpdatePresented: Binding<Bool> {
        Binding(
            get: {
                _ = appUpdateGeneration
                guard configuration.showsAppUpdatePrompt else { return false }
                // A required update waits for an active connection to end, so Disconnect stays reachable.
                return configuration.blocksConnectForAppUpdate
                    ? viewModel.connectState != .connected
                    : !configuration.appUpdatePromptDismissed
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
#if os(iOS)
        let link = Constants.appStoreLink
#else
        let link = Constants.downloadLinkMacOS
#endif
        try? ExternalLinkManager.shared.openExternalURL(urlString: link.rawValue)
    }

    var connectButtonDisabled: Bool {
        if configuration.blocksConnectForAppUpdate &&
            viewModel.connectState != .connected &&
            viewModel.connectState != .stop {
            return true
        }
        switch viewModel.connectState {
        case .connecting, .disconnecting, .noInternet:
            return true
        case .disconnected, .stop, .connected, .noSubscription:
            return false
        }
    }
}
