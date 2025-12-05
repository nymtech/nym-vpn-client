#if os(macOS)
import SwiftUI
import AppSettings
import ConnectionManager
import FeatureFlagsManager
import Constants
import MessageModels
import NymVPNRpc
import GRPCManager
import Theme
import UIComponents

public struct ProxyView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @EnvironmentObject private var grpcManager: GRPCManager
    @Binding private var path: NavigationPath

    @State private var proxyStatusLoading = true
    @State private var proxyIsOn: Bool = false
    @State private var proxyStatus: Socks5Status?

    @State private var isSnackbarDisplayed = false
    @State private var snackbarMessage: String?

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            VStack(spacing: 0) {
                subtitleSection()
                proxyStatusSection()
            }
            .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .snackbar(
            isDisplayed: $isSnackbarDisplayed,
            message: SnackBarMessage(text: snackbarMessage ?? "", style: .info)
        )
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .task {
            do {
                proxyStatus = try await grpcManager.socks5Status()
            } catch GRPCError.daemonNotRunning {
                print("Daemon not running")
                proxyStatusLoading = false
            } catch GRPCError.invalidData {
                print("Invalid data")
                proxyStatusLoading = false
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
        .onChange(of: proxyStatus) { status in
            let isOn = switch status?.state {
            case .none, .some(.disabled), .some(.error):
                false
            case .some(.idle), .some(.connected):
                true
            }

            proxyIsOn = isOn
            proxyStatusLoading = false
        }
        .onChange(of: proxyIsOn) { isOn in
            guard !proxyStatusLoading else { return }
            print("Proxy is \(isOn ? "on" : "off")!")
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

// MARK: - Views -
private extension ProxyView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.proxy.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func subtitleSection() -> some View {
        Text("proxy.subtitle".localizedString)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.gray1)
            .padding(.vertical, 24)
    }

    func proxyStatusSection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: $proxyIsOn,
                        isDisabled: connectionManager.currentTunnelStatus != .connected,
                        action: { _ in
                            guard connectionManager.currentTunnelStatus == .connected else { return }
                        }
                    )
                ),
                title: "proxy.status.title".localizedString,
                subtitle: "proxy.status.subtitle".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                vpnAndProxyStatusDetails()
            }
        )
    }

    func vpnAndProxyStatusDetails() -> some View {
        VStack {
            let statusButtonConfig = StatusButtonConfig(
                tunnelStatus: connectionManager.currentTunnelStatus,
                hasInternet: true
            )
            detailsSection(
                title: "proxy.vpnStatus".localizedString,
                details: statusButtonConfig.title,
                color: vpnStatusColor()
            )
            .padding(.bottom, 12)

            Divider()
                .frame(height: 1)
                .overlay(NymColor.gray2)

            detailsSection(
                title: "proxy.proxyStatus".localizedString,
                details: proxyStatusText(),
                color: proxyStatusColor()
            )
            .padding(.vertical, 12)

            Divider()
                .frame(height: 1)
                .overlay(NymColor.gray2)

            detailsSection(
                title: "proxy.activeConnections".localizedString,
                details: proxyActiveConnectionsText(),
                color: NymColor.primary
            )
            .padding(.top, 12)
        }
        .padding(.vertical, 16)
    }

    func detailsSection(title: String, details: String, color: Color) -> some View {
        HStack(alignment: .firstTextBaseline) {
            Text(title)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
            Text(details)
                .foregroundStyle(color)
                .textStyle(.Body.Medium.bold)
        }
    }

    func vpnStatusColor() -> Color {
        switch connectionManager.currentTunnelStatus {
        case .connected:
            NymColor.action
        case .disconnected:
            NymColor.error
        default:
            NymColor.warning
        }
    }

    func proxyStatusText() -> String {
        if proxyStatusLoading {
            "proxy.proxyStatus.loading".localizedString
        } else {
            switch proxyStatus?.state {
            case .none, .some(.disabled), .some(.error):
                "proxy.proxyStatus.disabled".localizedString
            case .some(.idle), .some(.connected):
                "proxy.proxyStatus.connected".localizedString
            }
        }
    }

    func proxyStatusColor() -> Color {
        if proxyStatusLoading {
            NymColor.primary
        } else {
            switch proxyStatus?.state {
            case .none, .some(.disabled), .some(.error):
                NymColor.error
            case .some(.idle), .some(.connected):
                NymColor.action
            }
        }
    }

    func proxyActiveConnectionsText() -> String {
        switch proxyStatus?.activeConnections {
        case .none:
            "0"
        case let .some(connections):
            "\(connections)"
        }
    }
}

// MARK: - Actions -
private extension ProxyView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}

#endif
