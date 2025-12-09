#if os(macOS)
import SwiftUI
import AppSettings
import ConnectionManager
import GRPCManager
import ImpactGenerator
import MessageModels
import NymLogger
import NymVPNRpc
import Theme

@MainActor public final class ProxyViewModel: ObservableObject {

    let defaultSocks5ProxyListenAddress = "127.0.0.1:1080"
    let defaultHttpRpcProxyListenAddress = "127.0.0.1:8545"

    private let appSettings: AppSettings

    @ObservedObject var connectionManager: ConnectionManager
    @ObservedObject var grpcManager: GRPCManager

    @Binding private var path: NavigationPath

    @Published var proxyStatus: Socks5Status?
    @Published var proxyIsOn = false
    @Published var proxyStatusLoading = false

    @Published var isSnackbarDisplayed = false
    @Published var snackbarMessage: String?

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
            proxyStatusLoading = false
        } catch {
            proxyStatusLoading = false
            withAnimation {
                guard !isSnackbarDisplayed else { return }
                proxyStatusLoading = false
                snackbarMessage = "proxy.connectionError".localizedString
                isSnackbarDisplayed = true
                Task { @MainActor in
                    try? await Task.sleep(for: .seconds(3))
                    isSnackbarDisplayed = false
                }
            }
        }
    }

    func toggleProxy() async {
        guard connectionManager.currentTunnelStatus == .connected else {
            if !isSnackbarDisplayed {
                isSnackbarDisplayed = true
                snackbarMessage = "proxy.snackbar.connectionRequired".localizedString
                Task { @MainActor in
                    try? await Task.sleep(for: .seconds(3))
                    isSnackbarDisplayed = false
                }
            }
            return
        }

        do {
            proxyStatusLoading = true
            if proxyIsOn {
                try await grpcManager.disableSocks5()
            } else {
                try await grpcManager.enableSocks5(
                    socks5Settings: Socks5Settings(listenAddress: defaultSocks5ProxyListenAddress),
                    httpRpcSettings: HttpRpcSettings(listenAddress: defaultHttpRpcProxyListenAddress),
                    exitPoint: connectionManager.connectionConfig.exitPoint
                )
            }

            proxyStatus = try await grpcManager.socks5Status()
            proxyIsOn = isProxyOn()
            proxyStatusLoading = false
        } catch {
            proxyStatusLoading = false
            withAnimation {
                guard !isSnackbarDisplayed else { return }
                proxyStatusLoading = false
                snackbarMessage = "proxy.snackbar.connectionFailed".localizedString
                isSnackbarDisplayed = true
                Task { @MainActor in
                    try? await Task.sleep(for: .seconds(3))
                    isSnackbarDisplayed = false
                }
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
}

#endif
