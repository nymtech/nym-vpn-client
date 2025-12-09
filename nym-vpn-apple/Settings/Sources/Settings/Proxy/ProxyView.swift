#if os(macOS)
import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import MessageModels
import NymVPNRpc
import GRPCManager
import Theme
import UIComponents

public struct ProxyView: View {
    @StateObject private var viewModel: ProxyViewModel

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
            isDisplayed: $viewModel.isSnackbarDisplayed,
            message: SnackBarMessage(text: viewModel.snackbarMessage ?? "", style: .info)
        )
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .task {
            await viewModel.loadSocks5Status()
        }
        .onChange(of: viewModel.proxyStatus) { status in
            let isOn = switch status?.state {
            case .none, .some(.disabled), .some(.error):
                false
            case .some(.idle), .some(.connected):
                true
            }

            viewModel.proxyIsOn = isOn
            viewModel.proxyStatusLoading = false
        }
        .onChange(of: viewModel.proxyIsOn) { isOn in
            guard !viewModel.proxyStatusLoading else { return }
            print("Proxy is \(isOn ? "on" : "off")!")
        }
    }

    public init(viewModel: ProxyViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }
}

// MARK: - Views -
private extension ProxyView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.proxy.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
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
                        isOn: $viewModel.proxyIsOn,
                        isDisabled: viewModel.connectionManager.currentTunnelStatus != .connected,
                        isInteractiveWhenDisabled: true,
                        action: { _ in viewModel.toggleProxy() }
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
    
    @ViewBuilder func proxySettingsList() -> some View {
        VStack {
            SettingsListItem(viewModel: <#T##SettingsListItemViewModel#>)
        }
    }
}

private extension ProxyView {
    func vpnAndProxyStatusDetails() -> some View {
        VStack {
            let statusButtonConfig = StatusButtonConfig(
                tunnelStatus: viewModel.connectionManager.currentTunnelStatus,
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
        switch viewModel.connectionManager.currentTunnelStatus {
        case .connected:
            NymColor.action
        case .disconnected:
            NymColor.error
        default:
            NymColor.warning
        }
    }

    func proxyStatusText() -> String {
        if viewModel.proxyStatusLoading {
            "proxy.proxyStatus.loading".localizedString
        } else {
            switch viewModel.proxyStatus?.state {
            case .none, .some(.disabled), .some(.error):
                "proxy.proxyStatus.disabled".localizedString
            case .some(.idle), .some(.connected):
                "proxy.proxyStatus.connected".localizedString
            }
        }
    }

    func proxyStatusColor() -> Color {
        if viewModel.proxyStatusLoading {
            NymColor.primary
        } else {
            switch viewModel.proxyStatus?.state {
            case .none, .some(.disabled), .some(.error):
                NymColor.error
            case .some(.idle), .some(.connected):
                NymColor.action
            }
        }
    }

    func proxyActiveConnectionsText() -> String {
        switch viewModel.proxyStatus?.activeConnections {
        case .none:
            "0"
        case let .some(connections):
            "\(connections)"
        }
    }
}

#endif
