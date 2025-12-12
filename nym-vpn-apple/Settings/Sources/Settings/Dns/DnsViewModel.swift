import SwiftUI
import AppSettings

@MainActor public final class DnsViewModel: ObservableObject {

    private let appSettings: AppSettings

    @Binding private var path: NavigationPath
    
    @Published var isDefaultDnsDisplayed = false
    
    @Published var isCustomDnsEnabled = false

    @Published var isSnackbarDisplayed = false
    @Published var snackbarMessage: String?

    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings
    ) {
        _path = path
        self.appSettings = appSettings
    }
}

extension DnsViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

private extension DnsViewModel {
}
