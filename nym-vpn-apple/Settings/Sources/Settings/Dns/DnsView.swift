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

    @State private var isCustomDnsHovered = false

    @FocusState private var isCustomDnsTextFieldFocused: Bool
    @State private var isCustomDnsTextFieldHovered = false
    @State private var isCustomDnsTextFieldDirty = false

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
            .onTapGesture {
                isCustomDnsTextFieldFocused = false
            }
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
        .overlay {
            saveChangesOverlay()
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
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack(discardChanges: false) })
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
                        isDisabled: viewModel.customDns.isEmpty,
                        action: { _ in Task { await viewModel.toggleCustomDns() } }
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

    #if os(macOS)
    private static let dnsEntryHeight: CGFloat = 44
    #elseif os(iOS)
    private static let dnsEntryHeight: CGFloat = 28
    #endif

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

    #if os(macOS)
    @ViewBuilder
    func customDnsList() -> some View {
        VStack(spacing: 0) {
            if !viewModel.customDns.isEmpty {
                HStack {
                    Text("\("dns.custom.listTitle".localizedString) (\(viewModel.customDns.count)/\(viewModel.maxDnsEntries))")
                        .textStyle(.Body.Medium.regular)
                        .foregroundStyle(NymColor.primary)
                        .padding(.vertical, 12)
                    Spacer()
                }
                .padding(.horizontal, 16)
                Divider()
                    .frame(height: 1)
                    .overlay(NymColor.gray2)
                    .padding(.horizontal, 16)
            }
            List {
                ForEach(viewModel.customDns, id: \.self) { ip in
                    VStack {
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
                        .background(.clear)
                        .padding(.horizontal, 16)
                        Divider()
                            .frame(height: 1)
                            .overlay(NymColor.gray2)
                            .frame(maxWidth: .infinity)
                            .padding(.horizontal, 8)
                    }
                    .listRowSeparator(.hidden)
                }
                .onMove { from, to in
                    viewModel.customDns.move(fromOffsets: from, toOffset: to)
                }
            }
            .listStyle(.plain)
            .background(.clear)
            .scrollContentBackground(.hidden)
            .scrollDisabled(true)
            .frame(maxWidth: .infinity)
            .frame(height: (DnsView.dnsEntryHeight * CGFloat(viewModel.customDns.count)) * 1.45)
        }
    }
    #elseif os(iOS)
    func customDnsList() -> some View {
        VStack(spacing: 0) {
            if !viewModel.customDns.isEmpty {
                HStack {
                    Text("\("dns.custom.listTitle".localizedString) (\(viewModel.customDns.count)/\(viewModel.maxDnsEntries))")
                        .textStyle(.Body.Medium.regular)
                        .foregroundStyle(NymColor.primary)
                        .padding(.vertical, 12)
                    Spacer()
                }
                .padding(.horizontal, 16)
                Divider()
                    .frame(height: 1)
                    .overlay(NymColor.gray2)
                    .padding(.horizontal, 16)
            }
            List {
                ForEach(viewModel.customDns, id: \.self) { ip in
                    VStack {
                        HStack {
                            HStack {
                                GenericImage(imageName: "dragIndicator")
                                    .frame(width: 20, height: 20)
                                    .foregroundStyle(NymColor.gray1)
                                Text(ip)
                            }
                            .padding(.leading, 16)
                            Spacer()
                            GenericImage(systemImageName: "trash")
                                .frame(width: 20, height: 20)
                                .foregroundStyle(NymColor.primary)
                                .onTapGesture {
                                    viewModel.deleteCustom(ipAddr: ip)
                                }
                                .padding(.trailing, 16)
                        }
                        .alignmentGuide(.listRowSeparatorLeading) { _ in 0 }
                    }
                    .frame(height: DnsView.dnsEntryHeight)
                    .listRowSeparatorTint(NymColor.gray2)
                    .listRowBackground(NymColor.elevation)
                }
                .onMove { from, to in
                    viewModel.customDns.move(fromOffsets: from, toOffset: to)
                }
                .listRowInsets(
                    EdgeInsets(
                        top: 0,
                        leading: 16,
                        bottom: 0,
                        trailing: 16
                    )
                )
            }
            .listStyle(.plain)
            .background(NymColor.elevation)
            .scrollContentBackground(.hidden)
            .scrollDisabled(true)
            .frame(maxWidth: .infinity)
            .frame(height: (DnsView.dnsEntryHeight * CGFloat(viewModel.customDns.count)) * 2)
        }
    }
    #endif

    @ViewBuilder
    func dnsTextFieldAndAddButton() -> some View {
        if viewModel.customDns.count < viewModel.maxDnsEntries {
            VStack {
                HStack(spacing: 16) {
                    dnsTextField()
                    dnsAddButton()
                }

                if let validationError = viewModel.customDnsValidationError,
                   !isCustomDnsTextFieldFocused,
                    isCustomDnsTextFieldDirty {
                    HStack {
                        Text(validationError)
                            .textStyle(.Body.Medium.regular)
                            .foregroundStyle(NymColor.error)
                        Spacer()
                    }
                }
            }
            .padding(16)
        }
    }

    @ViewBuilder
    func dnsTextField() -> some View {
        StrokeBorderView(
            strokeTitle: "dns.textfield.title".localizedString,
            strokeTitleLeftMargin: 60,
            isHovered: $isCustomDnsTextFieldHovered,
            strokeColor: viewModel.customDnsValidationError != nil
                && !isCustomDnsTextFieldFocused
                && isCustomDnsTextFieldDirty
                    ? NymColor.error
                    : NymColor.primary,
            backgroundColor: .clear
        ) {
            HStack {
                ZStack(alignment: .leading) {
                    TextField("", text: $viewModel.customDnsTextField)
                        .padding(.horizontal, 16)
                        .foregroundStyle(NymColor.primary)
                        .textFieldStyle(PlainTextFieldStyle())
                        .background(.clear)
                        .textStyle(.Body.Large.regular)
                        .focused($isCustomDnsTextFieldFocused)

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
            isCustomDnsTextFieldFocused = true
        }
        .onChange(of: viewModel.customDnsTextField) { newValue in
            if !newValue.isEmpty {
                isCustomDnsTextFieldDirty = true
            }
        }
    }

    @ViewBuilder
    func dnsAddButton() -> some View {
        GenericButton(
            title: "dns.button.add".localizedString,
            style: .primaryBorderOnly,
            isDisabled: viewModel.isAddButtonDisabled,
            isWidthExpanded: false
        )
        .onTapGesture {
            guard !viewModel.isAddButtonDisabled else { return }
            viewModel.add()
            isCustomDnsTextFieldDirty = false
        }
        .accessibilityAction {
            guard !viewModel.isAddButtonDisabled else { return }
            viewModel.add()
            isCustomDnsTextFieldDirty = false
        }
    }

    @ViewBuilder
    func dnsSaveChangesButton() -> some View {
        GenericButton(
            title: "dns.button.saveChanges".localizedString,
            isDisabled: viewModel.isSaveChangesButtonDisabled
        )        .padding(.horizontal, 16)
        .onTapGesture {
            Task {
                await viewModel.saveChanges()
            }
        }
        .accessibilityAction {
            Task {
                await viewModel.saveChanges()
            }
        }
    }

    @ViewBuilder
    func saveChangesOverlay() -> some View {
        if viewModel.isSaveChangesModalDisplayed {
            ActionDialogView(
                viewModel: ActionDialogViewModel(
                    isDisplayed: $viewModel.isSaveChangesModalDisplayed,
                    configuration: viewModel.saveChangesModalConfiguration,
                    impactGenerator: .shared
                )
            )
            .transition(.opacity)
            .animation(.easeInOut, value: viewModel.isSaveChangesModalDisplayed)
        }
    }
}
