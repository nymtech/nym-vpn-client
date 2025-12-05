import SwiftUI
import AppSettings
import ConnectionManager
import FeatureFlagsManager
import Constants
import MessageModels
import Theme
import UIComponents

public struct ProxyView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @Binding private var path: NavigationPath
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
                        isOn: $appSettings.isProxyEnabled,
                        isDisabled: connectionManager.currentTunnelStatus != .connected,
                        action: { _ in
                            guard connectionManager.currentTunnelStatus == .connected else { return }
                            appSettings.isProxyEnabled.toggle()
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
            HStack {
                Text("proxy.vpnStatus".localizedString)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Medium.regular)
                Spacer()

                let statusButtonConfig = StatusButtonConfig(
                    tunnelStatus: connectionManager.currentTunnelStatus,
                    hasInternet: true
                )
                Text(statusButtonConfig.title)
                    .foregroundStyle(vpnStatusColor())
                    .textStyle(.Body.Medium.bold)
            }

            Divider()
                .frame(height: 1)
                .overlay(NymColor.background)
                .padding(.vertical, 12)

            HStack {
                Text("proxy.proxyStatus".localizedString)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Medium.regular)
                Spacer()
                // TODO
                let statusButtonConfig = StatusButtonConfig(
                    tunnelStatus: connectionManager.currentTunnelStatus,
                    hasInternet: true
                )
                Text(statusButtonConfig.title)
                    .foregroundStyle(vpnStatusColor())
                    .textStyle(.Body.Medium.bold)
            }
            HStack {
            }
        }
        .padding(.vertical, 16)
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
}

// MARK: - Actions -
private extension ProxyView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
