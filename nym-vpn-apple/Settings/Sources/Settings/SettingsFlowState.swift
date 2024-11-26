import SwiftUI

public class SettingsFlowState: ObservableObject {
    @Published var presentedItem: SettingLink?

    @Binding var path: NavigationPath

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}
