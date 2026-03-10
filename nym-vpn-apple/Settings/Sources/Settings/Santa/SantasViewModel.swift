import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import NymLogger
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import Theme

@MainActor public final class SantasViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()
    @Binding private var path: NavigationPath

    let title = "🎅 Santa's menu 🎅"

    var actualEnv: String {
#if os(iOS)
        configurationManager.networkEnv?.current().nymNetwork.networkName ?? "unknown"
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
        appSettings: AppSettings,
        configurationManager: ConfigurationManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }
#elseif os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.grpcManager = grpcManager
        self.configurationManager = configurationManager
    }
#endif

    func changeEnvironment(to env: Env) {
        configurationManager.updateEnv(to: env)
        objectWillChange.send()
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    var logFilesSize: String {
        let fm = FileManager.default
        var totalSize: UInt64 = 0
        for type in LogFileType.allCases {
            guard let url = LogFileManager.logFileURL(logFileType: type),
                  let attrs = try? fm.attributesOfItem(atPath: url.path),
                  let size = attrs[.size] as? UInt64
            else { continue }
            totalSize += size
        }
        return ByteCountFormatter.string(fromByteCount: Int64(totalSize), countStyle: .file)
    }

#if os(macOS)
    func updateDaemonInfo() {
        Task {
            try? await grpcManager.version()
            Task { @MainActor in
                objectWillChange.send()
            }
        }
    }
#endif
}
