#if os(macOS)
import SwiftUI
import AppSettings
import ConnectionManager
import GRPCManager
import ImpactGenerator
import MessagesManager
import MessageModels
import NymLogger
import NymVPNRpc
import Theme

@MainActor public final class ProxyViewModel: ObservableObject {
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
        grpcManager: GRPCManager,
        messagesManager: MessagesManager
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
        } catch {
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

    func toggleProxy() {
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
        
        
    }
}
#endif
