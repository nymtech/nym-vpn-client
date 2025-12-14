import SwiftUI
import AppSettings

@MainActor public final class DnsViewModel: ObservableObject {
    private let appSettings: AppSettings
    let maxDnsEntries = 5

    @Binding private var path: NavigationPath

    @Published var isDefaultDnsDisplayed = false
    @Published var isCustomDnsEnabled = false
    @Published var ipAddresses: [String] = [
        "192.168.1.1",
        "10.0.0.1",
        "208.67.222.222",
        "208.67.220.220",
        "208.67.220.221"
    ]
    @Published var ipAddressTextField = ""

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
