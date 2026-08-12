#if os(macOS)
import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import NymVPNLib
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
                .scrollIndicators(.never)
            }
            .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .task {
            await viewModel.loadSocks5Status()
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
            .nymTextStyle(.bodyDefault)
            .foregroundStyle(Color.Nym.textSecondary)
            .padding(.vertical, 24)
    }

    func proxyStatusSection() -> some View {
        let proxyBinding = Binding<Bool>(
            get: { viewModel.proxyIsOn },
            set: { newValue in
                if viewModel.connectionManager.currentTunnelStatus == .connected {
                    viewModel.proxyIsOn = newValue
                    Task { await viewModel.toggleProxy() }
                } else if !viewModel.proxyStatusLoading {
                    Task { await viewModel.toggleProxy() }
                }
            }
        )
        let isDisabled = viewModel.connectionManager.currentTunnelStatus != .connected
        && viewModel.proxyStatusLoading

        return SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: proxyBinding,
                    isDisabled: isDisabled
                ),
                title: "proxy.status.title".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                proxyStatusDetails()
            }
        )
        .padding(.bottom, 16)
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
    func proxyStatusDetails() -> some View {
        VStack {
            detailsSection(
                title: "proxy.proxyStatus".localizedString,
                details: proxyStatusText(),
                color: proxyStatusColor()
            )
            .padding(.bottom, 12)
            Divider()
                .frame(height: 1)
                .overlay(Color.Nym.divider)
            detailsSection(
                title: "proxy.activeConnections".localizedString,
                details: proxyActiveConnectionsText(),
                color: Color.Nym.textPrimary
            )
            .padding(.top, 12)
        }
        .padding(.top, 8)
        .padding(.bottom, 16)
        .padding(.horizontal, 16)
    }

    func detailsSection(title: String, details: String, color: Color) -> some View {
        HStack(alignment: .firstTextBaseline) {
            Text(title)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
            Text(details)
                .foregroundStyle(color)
                .nymTextStyle(.bodyDefault)
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
            Color.Nym.textPrimary
        } else {
            switch viewModel.proxyStatus?.state {
            case .none, .some(.disabled), .some(.error):
                Color.Nym.error
            case .some(.idle), .some(.connected):
                Color.Nym.primary
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
            if viewModel.proxyIsOn {
                socksUrlSection()
                socksInstructionsSection()
            }
        }
        .padding(.bottom, 24)
    }

    func socksProxySection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "proxy.socks5.proxyTitle".localizedString,
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: true, isLast: !viewModel.proxyIsOn),
                action: { viewModel.copyListenAddress(for: .socks5, fullyQualified: false) }
            ),
            customContent: {
                VStack {
                    HStack {
                        Text(viewModel.socks5ProxyListenAddress.url)
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.Nym.textSecondary)
                        Spacer()
                        GenericImage(imageName: viewModel.socks5Copied ? "checkmarkSeeThrough" : "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
                .padding(.horizontal, 16)
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
                Text("ℹ️  \("proxy.socks5.instructions".localizedString)")
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 16)
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
                title: "proxy.socks5.proxyUrlTitle".localizedString,
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: false, isLast: false),
                action: { viewModel.copyListenAddress(for: .socks5, fullyQualified: true) }
            ),
            customContent: {
                VStack {
                    HStack {
                        Text(AttributedString(viewModel.socks5ProxyListenAddress.fullyQualified))
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.Nym.textSecondary)
                        Spacer()
                        GenericImage(imageName: viewModel.socks5CopiedFullyQualified ? "checkmarkSeeThrough" : "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
                .padding(.horizontal, 16)
            }
        )
    }

    func httpRpcProxySettings() -> some View {
        VStack(spacing: 0) {
            httpRpcProxySection()
            if viewModel.proxyIsOn {
                httpRpcUrlSection()
                httpRpcInstructionsSection()
            }
        }
    }

    func httpRpcProxySection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "proxy.httpRpc.proxyTitle".localizedString,
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: true, isLast: !viewModel.proxyIsOn),
                action: { viewModel.copyListenAddress(for: .httpRpc, fullyQualified: false) }
            ),
            customContent: {
                VStack {
                    HStack {
                        Text(viewModel.httpRpcProxyListenAddress.url)
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.Nym.textSecondary)
                        Spacer()
                        GenericImage(imageName: viewModel.isHttpRpcCopied ? "checkmarkSeeThrough" : "copy")
                            .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
                .padding(.horizontal, 16)
            }
        )
    }

    func httpRpcUrlSection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: "proxy.httpRpc.proxyUrlTitle".localizedString,
                systemImageName: "number",
                position: SettingsListItemPosition(isFirst: false, isLast: false),
                action: { viewModel.copyListenAddress(for: .httpRpc, fullyQualified: true) }
            ),
            customContent: {
                VStack {
                    HStack {
                        Text(AttributedString(viewModel.httpRpcProxyListenAddress.fullyQualified))
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.Nym.textSecondary)
                        Spacer()
                        GenericImage(
                            imageName: viewModel.isHttpRpcCopiedFullyQualified ? "checkmarkSeeThrough" : "copy"
                        )
                        .frame(width: 24, height: 24)
                    }
                }
                .padding(.bottom, 16)
                .padding(.horizontal, 16)
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
                Text("ℹ️  \("proxy.httpRpc.instructions".localizedString)")
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 16)
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
