import SwiftUI
import AccountPrefetchGates

public class SettingsFlowState: ObservableObject {
    @Published var presentedItem: SettingLink?

    @Binding var path: NavigationPath
    public var onSessionEvent: ((SessionEvent) -> Void)?

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}
