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
}
