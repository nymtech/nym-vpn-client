import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif
import Theme

public final class SantasViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    @Binding private var path: NavigationPath

    let title = "🎅 Santa's menu 🎅"

    var actualEnv: String {
#if os(iOS)
        let result = try? currentEnvironment()
        return "\(result?.nymNetwork.networkName ?? "Cannot determine network name")"
#elseif os(macOS)
        grpcManager.networkName ?? "Restart app to see"
#endif
    }

    var currentAppEnv: String {
        appSettings.currentEnv
    }

    var envs: [Env] {
        Env.allCases
    }

    var libVersion: String {
#if os(iOS)
        AppVersionProvider.libVersion
#elseif os(macOS)
        grpcManager.daemonVersion
#endif
    }

#if os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }
#elseif os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        grpcManager: GRPCManager = .shared
    ) {
        _path = path
        self.appSettings = appSettings
        self.grpcManager = grpcManager
        self.configurationManager = configurationManager
    }
#endif

    func changeEnvironment(to env: Env) {
        configurationManager.updateEnv(to: env)
        navigateBack()
    }

    func clearEntryGateway() {
        appSettings.entryGateway = nil
        navigateBack()
    }

    func clearExitGateway() {
        appSettings.exitGateway = nil
        navigateBack()
    }

    func clearBothGateways() {
        appSettings.entryGateway = nil
        appSettings.exitGateway = nil
        navigateBack()
    }

    func pasteEntryGateway() {
        let gateway: String?
#if os(iOS)
        gateway = UIPasteboard.general.string
#elseif os(macOS)
        gateway = NSPasteboard.general.string(forType: .string)
#endif
        appSettings.entryGateway = gateway
        navigateBack()
    }

    func pasteExitGateway() {
        let gateway: String?
#if os(iOS)
        gateway = UIPasteboard.general.string
#elseif os(macOS)
        gateway = NSPasteboard.general.string(forType: .string)
#endif
        appSettings.exitGateway = gateway
        navigateBack()
    }

    func entryGatewayString() -> String {
        appSettings.entryGateway ?? "None"
    }

    func exitGatewayString() -> String {
        appSettings.exitGateway ?? "None"
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
