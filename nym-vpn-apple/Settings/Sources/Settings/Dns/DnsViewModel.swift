#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import NymVPNRpc
import GRPCManager
#endif
import SwiftUI
import AppSettings
import Network
import UIComponents

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
    public var isAddButtonDisabled: Bool {
        !isIPAddress(customDnsTextField)
        || customDns.contains(customDnsTextField)
        || customDns.count == maxDnsEntries
    }
    public var customDnsValidationError: String? {
        if !isIPAddress(customDnsTextField) {
            "dns.textfield.invalid".localizedString
        } else {
            nil
        }
    }

    public var isSaveChangesButtonDisabled: Bool { customDns == appSettings.customDns }
    @Published var isSaveChangesModalDisplayed = false

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
        self.customDns = appSettings.customDns
    }
    #elseif os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings
    ) {
        _path = path
        self.appSettings = appSettings
        self.customDns = appSettings.customDns
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

        appSettings.customDns = customDns
        #if os(macOS)
        do {
            try await grpcManager.setCustomDns(dnsServers: customDns)
        } catch {
            withAnimation {
                guard !isSnackbarDisplayed else { return }
                snackbarMessage = "generalNymError.somethingWentWrong".localizedString
                isSnackbarDisplayed = true
                Task { @MainActor in
                    try? await Task.sleep(for: .seconds(3))
                    isSnackbarDisplayed = false
                }
            }
            return
        }
        #endif

        appSettings.shouldReconnect = true

        withAnimation {
            guard !isSnackbarDisplayed else { return }
            snackbarMessage = "dns.snackbar.saved".localizedString
            isSnackbarDisplayed = true
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                isSnackbarDisplayed = false
            }
        }
    }

    func toggleCustomDns() async {
        guard !appSettings.customDns.isEmpty else { return }

        isCustomDnsEnabled.toggle()
        appSettings.isCustomDnsEnabled = isCustomDnsEnabled
        #if os(macOS)
        do {
            try await grpcManager.setEnableCustomDns(enable: isCustomDnsEnabled)
        } catch {
            isCustomDnsEnabled.toggle()
            withAnimation {
                guard !isSnackbarDisplayed else { return }
                snackbarMessage = "generalNymError.somethingWentWrong".localizedString
                isSnackbarDisplayed = true
                Task { @MainActor in
                    try? await Task.sleep(for: .seconds(3))
                    isSnackbarDisplayed = false
                }
            }
        }
        #endif
    }
}

private extension DnsViewModel {
    func isIPAddress(_ string: String) -> Bool {
        IPv4Address(string) != nil || IPv6Address(string) != nil
    }
}
