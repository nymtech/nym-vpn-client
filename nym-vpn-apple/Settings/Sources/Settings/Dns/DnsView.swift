import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import MessageModels
import NymVPNRpc
import GRPCManager
import Theme
import UIComponents

public struct DnsView: View {
    @StateObject private var viewModel: DnsViewModel

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    subtitleSection()
                    if viewModel.isDefaultDnsDisplayed {
                        defaultDnsSection()
                    }

                    customDnsSection()
                }
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            .padding(.horizontal, 16)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init(viewModel: DnsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }
}

// MARK: - Views -
private extension DnsView {
    func navbar() -> some View {
        CustomNavBar(
            title: "dns.title".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    func subtitleSection() -> some View {
        VStack(alignment: .leading, spacing: 24) {
            Text("dns.subtitle".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)

            Text("dns.viewDefault".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
                .underline()
                .onTapGesture {
                    withAnimation {
                        viewModel.isDefaultDnsDisplayed.toggle()
                    }
                }
                .accessibilityAction {
                    withAnimation {
                        viewModel.isDefaultDnsDisplayed.toggle()
                    }
                }
        }
    }

    func defaultDnsSection() -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text("dns.default.title".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            // TODO ForEach
            Text("• 192.0.2.44")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192.0.2.45")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192.0.2.46")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192.0.2.48")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192.0.2.44")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192.0.2.45")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192.0.2.46")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Text("• 192:0::2::48::0")
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
        }
    }

    func customDnsSection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    viewModel: ToggleViewModel(
                        isOn: $viewModel.isCustomDnsEnabled,
                        action: { _ in }
                    )
                ),
                title: "dns.custom.title".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                customDnsInstructionsAndList()
            }
        )
    }

    func customDnsInstructionsAndList() -> some View {
        VStack {
            HStack {
                Text("dns.custom.instructions".localizedString)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)
                Spacer()
            }
        }
        .padding(.horizontal, 16)
        .padding(.bottom, 16)
    }
}
