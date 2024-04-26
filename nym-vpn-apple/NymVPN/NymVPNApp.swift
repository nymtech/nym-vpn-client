import SwiftUI
import AppSettings
import Home
import SentryManager
import Theme

@main
struct NymVPNApp: App {
    init() {
        setup()
    }

    var body: some Scene {
        WindowGroup {
//            GeometryReader { proxy in
                NavigationStack {
                    HomeView(viewModel: HomeViewModel(selectedNetwork: .mixnet5hop))
                }
//            }
            .environmentObject(AppSettings.shared)
        }
    }
}

private extension NymVPNApp {
    func setup() {
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
    }
}
