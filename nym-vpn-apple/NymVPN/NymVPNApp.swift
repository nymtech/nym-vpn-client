import SwiftUI
import AppSettings
import KeyboardManager
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
            NavigationStack {
                HomeView(viewModel: HomeViewModel(selectedNetwork: .mixnet5hop))
            }
            .onAppear {
                configureScreenSize()
            }
            .environmentObject(AppSettings.shared)
            .environmentObject(KeyboardManager.shared)
        }
    }
}

private extension NymVPNApp {
    func setup() {
        ThemeConfiguration.setup()
        SentryManager.shared.setup()
    }

    func configureScreenSize() {
        guard let screenSize = UIScreen.current?.bounds.size,
              screenSize.width <= 375 && screenSize.height <= 667,
              AppSettings.shared.isSmallScreen != true
        else {
            return
        }
        AppSettings.shared.isSmallScreen = true
    }
}
