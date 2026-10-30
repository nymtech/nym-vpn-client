#if os(macOS)
import GRPCManager
#endif
import SwiftUI
import AppSettings
import ConnectionTypes
import ConnectionManager
import Network
import UIComponents
import ExternalLinkManager
import Constants
import SnackbarManager

@MainActor public final class DnsViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let connectionManager: ConnectionManager
    let maxDnsEntries = 5
    let disallowedDnsEntries = ["0.0.0.0", "255.255.255.255"]

    @Binding private var path: NavigationPath

    @Published var defaultDns: [CustomDnsConfig.Server] = [
        .tls(address: "9.9.9.9", port: nil, serverName: "dns.quad9.net"),
        .tls(address: "149.112.112.112", port: nil, serverName: "dns.quad9.net"),
        .https(address: "9.9.9.9", port: nil, serverName: "dns.quad9.net", queryPath: "/dns-query"),
        .https(address: "149.112.112.112", port: nil, serverName: "dns.quad9.net", queryPath: "/dns-query"),
        .tls(address: "1.1.1.1", port: nil, serverName: "cloudflare-dns.com"),
        .tls(address: "1.0.0.1", port: nil, serverName: "cloudflare-dns.com"),
        .https(address: "1.1.1.1", port: nil, serverName: "cloudflare-dns.com", queryPath: "/dns-query"),
        .https(address: "1.0.0.1", port: nil, serverName: "cloudflare-dns.com", queryPath: "/dns-query")
    ]
    @Published var isDefaultDnsDisplayed = false

    @Published var customDns: [String] = []
    @Published var isCustomDnsEnabled = false

    var showsCustomDnsList: Bool { !customDns.isEmpty }

    @Published var customDnsTextField = ""
    public var isAddButtonDisabled: Bool {
        !isIPAddress(customDnsTextField)
        || customDns.contains(customDnsTextField)
        || customDns.count == maxDnsEntries
        || disallowedDnsEntries.contains(customDnsTextField)
    }
    public var customDnsValidationError: String? {
        if !isIPAddress(customDnsTextField) || disallowedDnsEntries.contains(customDnsTextField) {
            "dns.textfield.invalid".localizedString
        } else {
            nil
        }
    }

    public var isSaveChangesButtonDisabled: Bool { customDns == appSettings.customDns }
    @Published var isSaveChangesModalDisplayed = false

    #if os(macOS)
    private let grpcManager: GRPCManager

    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        grpcManager: GRPCManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.grpcManager = grpcManager
        self.customDns = appSettings.customDns
        self.isCustomDnsEnabled = appSettings.isCustomDnsEnabled
    }
    #elseif os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        connectionManager: ConnectionManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.customDns = appSettings.customDns
        self.isCustomDnsEnabled = appSettings.isCustomDnsEnabled
    }
    #endif
}

extension DnsViewModel {
    var saveChangesModalConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            systemIconImageName: "gearshape",
            titleLocalizedString: "dns.modals.saveChanges.title".localizedString,
            subtitleLocalizedString: "dns.modals.saveChanges.subtitle".localizedString,
            yesLocalizedString: "dns.button.saveChanges".localizedString,
            noLocalizedString: "dns.modals.saveChanges.discard".localizedString,
            isNoDestructive: true,
            yesAction: {
                Task {
                    await self.saveChanges()
                    self.isSaveChangesModalDisplayed = false
                    self.navigateBack(discardChanges: false)
                }
            },
            noAction: {
                self.navigateBack(discardChanges: true)
            },
            verticalButtonsLayout: true
        )
    }

    func navigateBack(discardChanges: Bool) {
        guard isSaveChangesButtonDisabled else {
            if discardChanges {
                if !path.isEmpty { path.removeLast() }
            } else {
                isSaveChangesModalDisplayed = true
            }
            return
        }

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

    func add() {
        guard !isAddButtonDisabled else { return }
        customDns.append(customDnsTextField)
        customDnsTextField = ""
    }

    func saveChanges() async {
        guard !isSaveChangesButtonDisabled else { return }
        if isCustomDnsEnabled && customDns.isEmpty {
            await toggleCustomDns()
        }

        connectionManager.setCustomDns(customDns)

        SnackbarManager.shared.enqueue(
            SnackbarItem(
                style: .confirmation,
                title: "dns.snackbar.saved".localizedString
            )
        )
    }

    func toggleCustomDns() async {
        guard !appSettings.customDns.isEmpty else { return }
        connectionManager.setCustomDnsEnabled(isCustomDnsEnabled)
    }

    func learnMore() {
        try? ExternalLinkManager.shared.openExternalURL(urlString: Constants.dnsLearnMoreURL.rawValue)
    }
}

private extension DnsViewModel {
    func isIPAddress(_ string: String) -> Bool {
        IPv4Address(string) != nil || IPv6Address(string) != nil
    }
}
