import SwiftUI

public class HomeFlowState: ObservableObject {
    @MainActor @Published var path = NavigationPath()
    @MainActor @Published var presentedItem: HomeLink?
    @MainActor @Published var coverItem: HomeLink?
}
