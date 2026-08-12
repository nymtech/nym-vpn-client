#if os(macOS)
import SwiftUI
import AppSettings
import ConnectionManager
import GRPCManager
import ImpactGenerator
import NymLogger
import SnackbarManager
import NymVPNLib
import Theme

@MainActor public final class ProxyViewModel: ObservableObject {

    public enum ProxyUrlType {
        case socks5
        case httpRpc
    }

    public struct ProxyUrl {
        public let type: ProxyUrlType
        public let url: String

        public init(type: ProxyUrlType, url: String) {
            self.type = type
            self.url = url
        }

        public var fullyQualified: String {
            switch self.type {
            case .socks5:
                "socks5h://\(url)"
            case .httpRpc:
                "http://\(url)?p=<your-provider-url>"
            }
        }
    }

    static let defaultSocks5ProxyListenAddress = "127.0.0.1:1080"
    static let defaultHttpRpcProxyListenAddress = "127.0.0.1:8545"

    private let appSettings: AppSettings

    @ObservedObject var connectionManager: ConnectionManager
    @ObservedObject var grpcManager: GRPCManager

    @Binding private var path: NavigationPath

    @Published var proxyStatus: Socks5Status?
    @Published var proxyStatusLoading = false

    @Published var proxyIsOn = false

    @Published var socks5ProxyListenAddress = ProxyUrl(type: .socks5, url: defaultSocks5ProxyListenAddress)
    @Published var socks5Copied = false
    @Published var socks5CopiedFullyQualified = false

    @Published var httpRpcProxyListenAddress = ProxyUrl(type: .httpRpc, url: defaultHttpRpcProxyListenAddress)
    @Published var isHttpRpcCopied = false
    @Published var isHttpRpcCopiedFullyQualified = false

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
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func loadSocks5Status() async {
        do {
            proxyStatus = try await grpcManager.socks5Status()
            proxyIsOn = isProxyOn()
            let socks5ProxyListenAddress = (
                proxyStatus?.socks5Settings.listenAddress
                ?? ProxyViewModel.defaultSocks5ProxyListenAddress
            ).replacingEmpty(with: ProxyViewModel.defaultSocks5ProxyListenAddress)
            self.socks5ProxyListenAddress = ProxyUrl(type: .socks5, url: socks5ProxyListenAddress)
            let httpRpcProxyListenAddress = (
                proxyStatus?.httpRpcSettings.listenAddress
                ?? ProxyViewModel.defaultHttpRpcProxyListenAddress
            ).replacingEmpty(with: ProxyViewModel.defaultHttpRpcProxyListenAddress)
            self.httpRpcProxyListenAddress = ProxyUrl(type: .httpRpc, url: httpRpcProxyListenAddress)
            proxyStatusLoading = false
        } catch {
            proxyStatusLoading = false
            SnackbarManager.shared.enqueue(
                SnackbarItem(
                    style: .negative,
                    title: "proxy.snackbar.connectionFailed".localizedString
                )
            )
        }
    }

    func toggleProxy() async {
        do {
            proxyStatusLoading = true
            if proxyIsOn {
                try await grpcManager.disableSocks5()
            } else {
                try await grpcManager.enableSocks5(
                    socks5Settings: Socks5Settings(listenAddress: socks5ProxyListenAddress.url),
                    httpRpcSettings: HttpRpcSettings(listenAddress: httpRpcProxyListenAddress.url),
                    exitPoint: connectionManager.connectionConfig.exitPoint
                )
            }

            await loadSocks5Status()
            if proxyIsOn {
                SnackbarManager.shared.enqueue(
                    SnackbarItem(
                        style: .confirmation,
                        title: "proxy.snackbar.successfullyEnabled".localizedString
                    )
                )
            }
        } catch {
            proxyStatusLoading = false
            SnackbarManager.shared.enqueue(
                SnackbarItem(
                    style: .negative,
                    title: "proxy.snackbar.connectionFailed".localizedString
                )
            )
        }
    }

    func copyListenAddress(for urlType: ProxyUrlType, fullyQualified: Bool) {
        let valueToCopy = switch (urlType, fullyQualified) {
        case (.socks5, let full):
            full ? socks5ProxyListenAddress.fullyQualified : socks5ProxyListenAddress.url
        case (.httpRpc, let full):
            full ? httpRpcProxyListenAddress.fullyQualified : httpRpcProxyListenAddress.url
        }

        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(valueToCopy, forType: .string)
        withAnimation {
            guard !copiedState(for: urlType, fullyQualified: fullyQualified) else { return }
            updateCopiedState(for: urlType, fullyQualified: fullyQualified, isCopied: true)

            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                updateCopiedState(for: urlType, fullyQualified: fullyQualified, isCopied: false)
            }
        }
    }
}

private extension ProxyViewModel {
    func isProxyOn() -> Bool {
        switch proxyStatus?.state {
        case .none, .some(.disabled), .some(.error):
            false
        case .some(.idle), .some(.connected):
            true
        }
    }

    func copiedState(for urlType: ProxyUrlType, fullyQualified: Bool) -> Bool {
        switch (urlType, fullyQualified) {
        case (.socks5, false):
            socks5Copied
        case (.socks5, true):
            socks5CopiedFullyQualified
        case (.httpRpc, false):
            isHttpRpcCopied
        case (.httpRpc, true):
            isHttpRpcCopiedFullyQualified
        }
    }

    func updateCopiedState(for urlType: ProxyUrlType, fullyQualified: Bool, isCopied: Bool) {
        switch (urlType, fullyQualified) {
        case (.socks5, false):
            socks5Copied = isCopied
        case (.socks5, true):
            socks5CopiedFullyQualified = isCopied
        case (.httpRpc, false):
            isHttpRpcCopied = isCopied
        case (.httpRpc, true):
            isHttpRpcCopiedFullyQualified = isCopied
        }
    }
}

extension String {
    func replacingEmpty(with defaultValue: String) -> String {
        self.isEmpty ? defaultValue : self
    }
}

#endif
