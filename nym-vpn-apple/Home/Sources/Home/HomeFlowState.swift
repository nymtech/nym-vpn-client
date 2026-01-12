import SwiftUI
import AppSettings
import Routes

@Observable
public class HomeFlowState: ObservableObject {
    var splashScreenDidDisplay = false

    var path = NavigationPath()
    var presentedItem: HomeLink?
    var coverItem: HomeLink?

    init() {
        self.path = NavigationPath()
        if !splashScreenDidDisplay {
            path.append(HomeLink.launchView)
        }
    }
}
