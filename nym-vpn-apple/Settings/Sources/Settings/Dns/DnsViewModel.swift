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

    
    @Published var defaultDns: [String] = [
        "9.9.9.9",
        "149.112.112.112",
        "2620:fe::fe",
        "2620:fe::fe:9",
        "1.1.1.1",
        "1.0.0.1",
        "2606:4700:4700::1111",
        "2606:4700:4700::1001"
    ]
    @Published var isDefaultDnsDisplayed = false

    @Published var customDns: [String] = []
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

    func loadDefaultDns() async {
        #if os(macOS)
        let dns = (try? await grpcManager.getDefaultDns()) ?? []
        defaultDns = dns.isEmpty ? defaultDns : dns
        #endif
    }

    func deleteCustom(ipAddr: String) {
        customDns.removeAll { $0 == ipAddr }
    }
    
    func saveChanges() {
        appSettings.isCustomDnsEnabled = isCustomDnsEnabled
        appSettings.customDns = customDns
        
        appSettings.shouldReconnect = true
    }
}

private extension DnsViewModel {
}
