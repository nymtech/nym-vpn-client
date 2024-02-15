import SwiftUI

public struct HopListViewModel {
    public let type: HopType
    public let isSmallScreen: Bool

    @Binding var path: NavigationPath

    public init(path: Binding<NavigationPath>, type: HopType, isSmallScreen: Bool = false) {
        self.type = type
        _path = path
        self.isSmallScreen = isSmallScreen
    }
}

extension HopListViewModel {
    func navigateHome() {
        path = .init()
    }
}
