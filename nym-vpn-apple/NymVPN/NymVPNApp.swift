import SwiftUI
import Home
import Theme
import AppSettings
import Tunnels

@main
struct NymVPNApp: App {

    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
            GeometryReader { proxy in
                NavigationStack {
                    HomeView(viewModel: HomeViewModel(screenSize: proxy.size, selectedNetwork: .mixnet))
                }
            }
            .environmentObject(AppSettings.shared)
            .environmentObject(TunnelsManager.shared)
        }
    }
}

private extension NymVPNApp {
    func setup() {
        ThemeConfiguration.setup()
    }
}
