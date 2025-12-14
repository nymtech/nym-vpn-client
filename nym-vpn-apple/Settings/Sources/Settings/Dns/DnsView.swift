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
        .onHover { newValue in
            isCustomDnsHovered = newValue
        }
    }

    private static let dnsEntryHeight: CGFloat = 44

    func customDnsInstructionsAndList() -> some View {
        VStack {
            HStack {
                Text("dns.custom.instructions".localizedString)
                    .textStyle(.Body.Medium.regular)
                    .foregroundStyle(NymColor.gray1)
                Spacer()
            }
            .padding(.horizontal, 16)

            List {
                ForEach(viewModel.ipAddresses, id: \.self) { ip in
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
                    }
                    .frame(height: DnsView.dnsEntryHeight)
                    .alignmentGuide(.listRowSeparatorLeading) { _ in 0 }
                    .background(.clear)
                }
                .onMove { from, to in
                    viewModel.ipAddresses.move(fromOffsets: from, toOffset: to)
                }
            }
            .background(.clear)
            .scrollContentBackground(.hidden)
            .scrollDisabled(true)
            .frame(maxWidth: .infinity)
            .frame(height: DnsView.dnsEntryHeight * CGFloat(viewModel.ipAddresses.count))
            dnsTextfield()
        }
        .padding(.bottom, 16)
    }

    @ViewBuilder
    func dnsTextfield() -> some View {
        HStack(spacing: 16) {
            StrokeBorderView(
                strokeTitle: "dns.textfield.title".localizedString,
                strokeTitleLeftMargin: 60,
                isHovered: $isCustomDnsTextFieldHovered,
                strokeColor: NymColor.primary,
                backgroundColor: NymColor.elevation,
                backgroundColorHover: NymColor.elevation.opacity(0.7)
            ) {
                HStack {
                    ZStack(alignment: .leading) {
                        TextField("", text: $viewModel.ipAddressTextField)
                            .foregroundStyle(NymColor.gray1)
                            .textFieldStyle(PlainTextFieldStyle())
                            .background(NymColor.elevation.opacity(isCustomDnsHovered ? 0.7 : 1))
                            .textStyle(.Body.Large.regular)
                            .focused($isIpAddressTextFieldFocused)
                            .padding(.horizontal, 16)

                        if viewModel.ipAddressTextField.isEmpty {
                            Text("dns.textfield.placeholder".localizedString)
                                .foregroundStyle(NymColor.gray1)
                                .textStyle(.Body.Large.regular)
                                .padding(.leading, 16)
                        }
                    }
                    Spacer()
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }
            .onTapGesture {
                isIpAddressTextFieldFocused = true
            }

            Text("dns.button.add".localizedString)
                .frame(height: 56)
                .padding(.horizontal, 16)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
                .background(NymColor.elevation.opacity(isCustomDnsAddButtonHovered ? 0.7 : 1.0))
                .cornerRadius(8)
                .overlay {
                    RoundedRectangle(cornerRadius: 8)
                        .inset(by: 0.5)
                        .stroke(NymColor.primary.opacity(isCustomDnsAddButtonHovered ? 0.7 : 1), lineWidth: 1)
                }
                .onHover { newValue in
                    isCustomDnsAddButtonHovered = newValue
                }
                .onTapGesture {
                    print("Add!")
                }
        }
        .padding(.horizontal, 16)
    }
}
