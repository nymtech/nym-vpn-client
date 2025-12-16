import SwiftUI
import AppSettings
import ConnectionManager
import Constants
import MessageModels
#if os(macOS)
import NymVPNRpc
import GRPCManager
#endif
import Theme
import UIComponents

public struct DnsView: View {
    @StateObject private var viewModel: DnsViewModel

    @FocusState private var isIpAddressTextFieldFocused: Bool
    @State private var isCustomDnsHovered = false
    @State private var isCustomDnsTextFieldHovered = false
    @State private var isCustomDnsAddButtonHovered = false

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
        .task {
            await viewModel.loadDefaultDns()
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
            ForEach(viewModel.defaultDns, id: \.self) { ip in
                Text("• \(ip)")
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)
            }
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
        .onHover { newValue in
            isCustomDnsHovered = newValue
        }
    }

    private static let dnsEntryHeight: CGFloat = 44

    func customDnsInstructionsAndList() -> some View {
        VStack(spacing: 0) {
            customDnsInstructions()
            customDnsList()
            dnsTextFieldAndAddButton()
            dnsSaveChangesButton()
        }
        .padding(.bottom, 16)
    }

    @ViewBuilder
    func customDnsInstructions() -> some View {
        HStack {
            Text("dns.custom.instructions".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
            Spacer()
        }
        .padding(.horizontal, 16)
    }

    @ViewBuilder
    func customDnsList() -> some View {
        List {
            ForEach(viewModel.customDns, id: \.self) { ip in
                HStack {
                    HStack {
                        GenericImage(imageName: "dragIndicator")
                            .frame(width: 20, height: 20)
                            .foregroundStyle(NymColor.gray1)
                        Text(ip)
                    }
                    Spacer()
                    GenericImage(systemImageName: "trash")
                        .frame(width: 20, height: 20)
                        .foregroundStyle(NymColor.primary)
                        .onTapGesture {
                            viewModel.deleteCustom(ipAddr: ip)
                        }
                }
                .frame(height: DnsView.dnsEntryHeight)
                .alignmentGuide(.listRowSeparatorLeading) { _ in 0 }
                .background(.clear)
            }
            .onMove { from, to in
                viewModel.customDns.move(fromOffsets: from, toOffset: to)
            }
        }
        .background(.clear)
        .scrollContentBackground(.hidden)
        .scrollDisabled(true)
        .frame(maxWidth: .infinity)
        .frame(height: DnsView.dnsEntryHeight * CGFloat(viewModel.customDns.count))
    }

    @ViewBuilder
    func dnsTextFieldAndAddButton() -> some View {
        HStack(spacing: 16) {
            dnsTextField()
            dnsAddButton()
        }
        .padding(16)
    }

    @ViewBuilder
    func dnsTextField() -> some View {
        StrokeBorderView(
            strokeTitle: "dns.textfield.title".localizedString,
            strokeTitleLeftMargin: 60,
            isHovered: $isCustomDnsTextFieldHovered,
            strokeColor: NymColor.primary,
            backgroundColor: .clear
        ) {
            HStack {
                ZStack(alignment: .leading) {
                    TextField("", text: $viewModel.customDnsTextField)
                        .padding(.horizontal, 16)
                        .foregroundStyle(NymColor.gray1)
                        .textFieldStyle(PlainTextFieldStyle())
                        .background(.clear)
                        .textStyle(.Body.Large.regular)
                        .focused($isIpAddressTextFieldFocused)

                    if viewModel.customDnsTextField.isEmpty {
                        Text("dns.textfield.placeholder".localizedString)
                            .padding(.leading, 16)
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body.Large.regular)
                            .background(.clear)
                    }
                }
                Spacer()
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .onTapGesture {
            isIpAddressTextFieldFocused = true
        }
    }

    @ViewBuilder
    func dnsAddButton() -> some View {
        GenericButton(
            title: "dns.button.add".localizedString,
            style: .primaryBorderOnly,
            isWidthExpanded: false
        )
        .onTapGesture {
            print("Add!")
        }
        .accessibilityAction {
            print("Add!")
        }
    }

    @ViewBuilder
    func dnsSaveChangesButton() -> some View {
        GenericButton(title: "dns.button.saveChanges".localizedString)
            .padding(.horizontal, 16)
            .onTapGesture {
                print("Save changes!")
            }
            .accessibilityAction {
                print("Save changes!")
            }
    }
}
