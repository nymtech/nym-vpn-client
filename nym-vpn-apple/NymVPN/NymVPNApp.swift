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
//                        .onChange(
//                            of: proxy.size,
//                            perform: { newSize in
//                                guard newSize != viewModel.screenSize else { return }
//                                viewModel.screenSize = newSize
//                            }
//                        )
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
