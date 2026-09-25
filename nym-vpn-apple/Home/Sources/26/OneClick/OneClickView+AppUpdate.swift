import SwiftUI
import ConfigurationManager

extension OneClickView {
    var appUpdatePresented: Binding<Bool> {
        Binding(
            get: {
                _ = appUpdateGeneration
                return configuration.showsAppUpdatePrompt &&
                    (configuration.blocksConnectForAppUpdate || !dismissedAppUpdate)
            },
            set: { presented in
                guard !presented else { return }
                if configuration.blocksConnectForAppUpdate {
                    appUpdateGeneration += 1
                } else {
                    dismissedAppUpdate = true
                }
            }
        )
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

    var appUpdateAlertTitle: String {
        if configuration.blocksConnectForAppUpdate {
            return "Update required"
        }
        return "Update available"
    }

    var appUpdateAlertMessage: String? {
        if configuration.blocksConnectForAppUpdate {
            return nil
        }
        return "A newer version of NymVPN is available. You can keep using this version, or update now."
    }
}
