import Combine
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

    private var cancellables = Set<AnyCancellable>()
    @Binding private var path: NavigationPath

    let title = "🎅 Santa's menu 🎅"

    @Published var isZknymEnabled = false

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

    var entryGateways: [String] {
        get {
            guard let decoded = try? JSONDecoder().decode([String].self, from: appSettings.santaEntryGatewaysData)
            else {
                return []
            }
            return decoded
        }
        set {
            appSettings.santaEntryGatewaysData = (try? JSONEncoder().encode(newValue)) ?? Data()
        }
    }

    var exitGateways: [String] {
        get {
            guard let decoded = try? JSONDecoder().decode([String].self, from: appSettings.santaExitGatewaysData)
            else {
                return []
            }
            return decoded
        }
        set {
            appSettings.santaExitGatewaysData = (try? JSONEncoder().encode(newValue)) ?? Data()
        }
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
        self.isZknymEnabled = appSettings.isZknymEnabled ?? false
        setupObservers()
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
        self.isZknymEnabled = appSettings.isZknymEnabled ?? false
        setupObservers()
    }
#endif

    func changeEnvironment(to env: Env) {
        configurationManager.updateEnv(to: env)
        objectWillChange.send()
    }

    func clearEntryGateway() {
        entryGateways.removeAll()
        objectWillChange.send()
    }

    func clearExitGateway() {
        exitGateways.removeAll()
        objectWillChange.send()
    }

    func pasteEntryGateway() {
        let gateway: String?
#if os(iOS)
        gateway = UIPasteboard.general.string
#elseif os(macOS)
        gateway = NSPasteboard.general.string(forType: .string)
#endif
        guard let gateway,
              gateway.count == 44,
              !entryGateways.contains(gateway)
        else {
            return
        }
        entryGateways.append(gateway)
        objectWillChange.send()
    }

    func pasteExitGateway() {
        let gateway: String?
#if os(iOS)
        gateway = UIPasteboard.general.string
#elseif os(macOS)
        gateway = NSPasteboard.general.string(forType: .string)
#endif
        guard let gateway,
              gateway.count == 44,
              !exitGateways.contains(gateway)
        else {
            return
        }
        exitGateways.append(gateway)
        objectWillChange.send()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

#if os(macOS)
    func updateDaemonInfo() {
        Task(priority: .background) {
            try? await grpcManager.version()
            Task { @MainActor in
                objectWillChange.send()
            }
        }
    }
#endif
}

private extension SantasViewModel {
    func setupObservers() {
        $isZknymEnabled
            .sink { [weak self] newValue in
                Task { @MainActor in
                    self?.appSettings.isZknymEnabled = newValue == true ? true : nil
                }
            }
            .store(in: &cancellables)
    }
}
