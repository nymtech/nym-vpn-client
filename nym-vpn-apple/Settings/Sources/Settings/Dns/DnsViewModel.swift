#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import SwiftUI
import AppSettings

@MainActor public final class DnsViewModel: ObservableObject {
    private let appSettings: AppSettings
    #if os(macOS)
    private let grpcManager: GRPCManager
    #endif
    let maxDnsEntries = 5

    @Binding private var path: NavigationPath

    @Published var defaultDns: [String] = []
    @Published var isDefaultDnsDisplayed = false

    @Published var customDns: [String] = [
        "192.168.1.1",
        "10.0.0.1",
        "208.67.222.222",
        "208.67.220.220",
        "208.67.220.221"
    ]
    @Published var isCustomDnsEnabled = false

    @Published var customDnsTextField = ""

    @Published var isSnackbarDisplayed = false
    @Published var snackbarMessage: String?

    #if os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        grpcManager: GRPCManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.grpcManager = grpcManager
    }
    #elseif os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings
    ) {
        _path = path
        self.appSettings = appSettings
    }
    #endif
}

extension DnsViewModel {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
    
    #if os(macOS)
    func loadDefaultDns() async {
        let dns = (try? await grpcManager.getDefaultDns()) ?? []
        defaultDns = dns.isEmpty ? defaultDns : dns
    }
    #endif
}

private extension DnsViewModel {
}
