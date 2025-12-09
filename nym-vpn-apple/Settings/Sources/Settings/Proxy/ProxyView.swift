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
                ScrollView {
                    subtitleSection()
                    proxyStatusSection()
                    proxySettingsList()
                }
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
        .padding(.bottom, 24)
    }

    @ViewBuilder
    func proxySettingsList() -> some View {
        VStack(spacing: 0) {
            socks5ProxySettings()
            httpRpcProxySettings()
        }
        .padding(.bottom, 16)
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

    func socks5ProxySettings() -> some View {
        VStack(spacing: 0) {
            socksProxySection()
            socksUrlSection()
            socksInstructionsSection()
        }
        .padding(.bottom, 24)
    }

    func socksProxySection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "SOCKS5 proxy (for apps)",
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: true, isLast: false),
                action: {}
            ),
            customContent: {
                VStack {
                    HStack {
                        Text("127.0.0.1:1080")
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body4.Medium.regular)
                        Spacer()
                        GenericImage(imageName: "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
            }
        )
    }

    @ViewBuilder
    func socksInstructionsSection() -> some View {
        let vm = SettingsListItemViewModel(
            accessory: .empty,
            title: "",
            position: SettingsListItemPosition(isFirst: false, isLast: true),
            action: {}
        )

        VStack(alignment: .center, spacing: 0) {
            HStack(spacing: 0) {
                Text("ℹ️  Add this to your browser's proxy settings to route traffic through the Nym mixnet")
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular.withSpacing(1.4))
            }
            .padding(16)
        }
        .frame(maxWidth: .infinity)
        .background {
            UnevenRoundedRectangle(
                topLeadingRadius: vm.topRadius,
                bottomLeadingRadius: vm.bottomRadius,
                bottomTrailingRadius: vm.bottomRadius,
                topTrailingRadius: vm.topRadius
            )
            .fill(vm.type.backgroundColor)
        }
        .overlay {
            UnevenRoundedRectangle(
                topLeadingRadius: vm.topRadius,
                bottomLeadingRadius: vm.bottomRadius,
                bottomTrailingRadius: vm.bottomRadius,
                topTrailingRadius: vm.topRadius
            )
            .stroke(vm.type.strokeColor, lineWidth: 1)
        }
        .clipShape(
            UnevenRoundedRectangle(
                topLeadingRadius: vm.topRadius,
                bottomLeadingRadius: vm.bottomRadius,
                bottomTrailingRadius: vm.bottomRadius,
                topTrailingRadius: vm.topRadius
            )
        )
    }

    func socksUrlSection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "SOCKS5 URL (for apps)",
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: false, isLast: false),
                action: {}
            ),
            customContent: {
                VStack {
                    HStack {
                        Text(AttributedString("socks5h://127.0.0.1:1080"))
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body4.Medium.regular)
                        Spacer()
                        GenericImage(imageName: "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
            }
        )
    }

    func httpRpcProxySettings() -> some View {
        VStack(spacing: 0) {
            httpRpcProxySection()
            httpRpcUrlSection()
            httpRpcInstructionsSection()
        }
    }

    func httpRpcProxySection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "HTTP RPC proxy (for wallets)",
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: true, isLast: false),
                action: {}
            ),
            customContent: {
                VStack {
                    HStack {
                        Text("127.0.0.1:8545")
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body4.Medium.regular)
                        Spacer()
                        GenericImage(imageName: "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
            }
        )
    }

    func httpRpcUrlSection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "HTTP RPC URL (for wallets)",
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: false, isLast: false),
                action: {}
            ),
            customContent: {
                VStack {
                    HStack {
                        Text(AttributedString("http://127.0.0.1:8545?p=<your-provider-url>"))
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body4.Medium.regular)
                        Spacer()
                        GenericImage(imageName: "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
            }
        )
    }

    @ViewBuilder
    func httpRpcInstructionsSection() -> some View {
        let vm = SettingsListItemViewModel(
            accessory: .empty,
            title: "",
            position: SettingsListItemPosition(isFirst: false, isLast: true),
            action: {}
        )

        VStack(alignment: .center, spacing: 0) {
            HStack(spacing: 0) {
                Text("ℹ️  Use this in MetaMask or other Web3 wallets to make RPC calls through the Nym mixnet. Replace <your-provider-url> with your actual RPC endpoint.")
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular.withSpacing(1.4))
            }
            .padding(16)
        }
        .frame(maxWidth: .infinity)
        .background {
            UnevenRoundedRectangle(
                topLeadingRadius: vm.topRadius,
                bottomLeadingRadius: vm.bottomRadius,
                bottomTrailingRadius: vm.bottomRadius,
                topTrailingRadius: vm.topRadius
            )
            .fill(vm.type.backgroundColor)
        }
        .overlay {
            UnevenRoundedRectangle(
                topLeadingRadius: vm.topRadius,
                bottomLeadingRadius: vm.bottomRadius,
                bottomTrailingRadius: vm.bottomRadius,
                topTrailingRadius: vm.topRadius
            )
            .stroke(vm.type.strokeColor, lineWidth: 1)
        }
        .clipShape(
            UnevenRoundedRectangle(
                topLeadingRadius: vm.topRadius,
                bottomLeadingRadius: vm.bottomRadius,
                bottomTrailingRadius: vm.bottomRadius,
                topTrailingRadius: vm.topRadius
            )
        )
    }
}

#endif
