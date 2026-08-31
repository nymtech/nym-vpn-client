import SwiftUI
import AppSettings
import ConnectionManager
import Constants
#if os(macOS)
import NymVPNLib
import GRPCManager
#endif
import Theme
import UIComponents
import Device
#if os(iOS)
import KeyboardManager
#endif

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
#if os(iOS)
            KeyboardHostView(bottomSafeAreaInset: 0) {
                scrollViewContent()
            }
#elseif os(macOS)
            scrollViewContent()
#endif
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
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

    @ViewBuilder
    func scrollViewContent() -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                subtitleSection()
                if viewModel.isDefaultDnsDisplayed {
                    defaultDnsSection()
                }

                customDnsSection()
                learnMoreLink()
            }
        }
        .scrollIndicators(.never)
        .frame(maxWidth: MagicNumbers.maxWidth)
        .padding(.horizontal, 16)
        .onTapGesture {
            isCustomDnsTextFieldFocused = false
        }
    }

    func subtitleSection() -> some View {
        VStack(alignment: .leading, spacing: 24) {
            Text("dns.subtitle".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)

            Text("dns.viewDefault".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
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
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            ForEach(viewModel.defaultDns, id: \.self) { ip in
                Text("• \(ip)")
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textSecondary)
            }
        }
    }

    func customDnsSection() -> some View {
        SettingsListItemCustomContent(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(
                    isOn: $viewModel.isCustomDnsEnabled,
                    isDisabled: viewModel.customDns.isEmpty
                ),
                title: "dns.custom.title".localizedString,
                position: .init(isFirst: true, isLast: true),
                action: {}
            ),
            customContent: {
                customDnsInstructionsAndList()
            }
        )
        .onChange(of: viewModel.isCustomDnsEnabled) {
            Task { await viewModel.toggleCustomDns() }
        }
        .onHover { newValue in
            isCustomDnsHovered = newValue
        }
    }

    private static let dnsEntryHeight: CGFloat = Device.isMacOS ? 44 : 28

    func customDnsInstructionsAndList() -> some View {
        VStack(spacing: 0) {
            customDnsInstructions()
            if viewModel.showsCustomDnsList {
                customDnsList()
            }
            dnsTextFieldAndAddButton()
            dnsSaveChangesButton()
        }
        .padding(.bottom, 16)
    }

    @ViewBuilder
    func customDnsInstructions() -> some View {
        HStack {
            Text("\("dns.custom.instructions1".localizedString) ⚠️ \("dns.custom.instructions2".localizedString)")
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
            Spacer()
        }
        .padding(.horizontal, 16)
    }

    #if os(macOS)
    @ViewBuilder
    func customDnsList() -> some View {
        VStack(spacing: 0) {
            dnsListTitle()
            List {
                ForEach(viewModel.customDns, id: \.self) { ip in
                    VStack {
                        HStack {
                            HStack {
                                GenericImage(imageName: "dragIndicator")
                                    .frame(width: 20, height: 20)
                                    .foregroundStyle(Color.Nym.textSecondary)
                                Text(ip)
                            }
                            Spacer()
                            GenericImage(systemImageName: "trash")
                                .frame(width: 20, height: 20)
                                .foregroundStyle(Color.Nym.textPrimary)
                                .onTapGesture {
                                    viewModel.deleteCustom(ipAddr: ip)
                                }
                        }
                        .frame(height: DnsView.dnsEntryHeight)
                        .background(.clear)
                        .padding(.horizontal, 16)
                        Divider()
                            .frame(height: 1)
                            .overlay(Color.Nym.divider)
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
        .clipped()
    }
    #elseif os(iOS)
    func customDnsList() -> some View {
        VStack(spacing: 0) {
            dnsListTitle()
            List {
                ForEach(viewModel.customDns, id: \.self) { ip in
                    VStack {
                        HStack {
                            HStack {
                                GenericImage(imageName: "dragIndicator")
                                    .frame(width: 20, height: 20)
                                    .foregroundStyle(Color.Nym.textSecondary)
                                Text(ip)
                            }
                            .padding(.leading, 16)
                            Spacer()
                            GenericImage(systemImageName: "trash")
                                .frame(width: 20, height: 20)
                                .foregroundStyle(Color.Nym.textPrimary)
                                .onTapGesture {
                                    viewModel.deleteCustom(ipAddr: ip)
                                }
                                .padding(.trailing, 16)
                        }
                        .alignmentGuide(.listRowSeparatorLeading) { _ in 0 }
                    }
                    .frame(height: DnsView.dnsEntryHeight)
                    .listRowSeparatorTint(Color.Nym.divider)
                    .listRowBackground(Color.Nym.surface)
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
            .background(Color.Nym.surface)
            .scrollContentBackground(.hidden)
            .scrollDisabled(true)
            .frame(maxWidth: .infinity)
            .frame(height: (DnsView.dnsEntryHeight * CGFloat(viewModel.customDns.count)) * 2)
        }
        .clipped()
    }
    #endif

    @ViewBuilder
    func dnsListTitle() -> some View {
        if !viewModel.customDns.isEmpty {
            HStack {
                Text("\("dns.custom.listTitle".localizedString) (\(viewModel.customDns.count)/\(viewModel.maxDnsEntries))")
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .padding(.vertical, 12)
                Spacer()
            }
            .padding(.horizontal, 16)
            Divider()
                .frame(height: 1)
                .overlay(Color.Nym.divider)
                .padding(.horizontal, 16)
        }
    }

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
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.Nym.error)
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
            isHovered: $isCustomDnsTextFieldHovered,
            strokeColor: viewModel.customDnsValidationError != nil
                && !isCustomDnsTextFieldFocused
                && isCustomDnsTextFieldDirty
                    ? Color.Nym.error
                    : Color.Nym.textPrimary,
            backgroundColor: Color.Nym.surface
        ) {
            HStack {
                ZStack(alignment: .leading) {
                    TextField("", text: $viewModel.customDnsTextField)
                        .padding(.horizontal, 16)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .textFieldStyle(PlainTextFieldStyle())
                        .background(.clear)
                        .nymTextStyle(.bodyLarge)
                        .focused($isCustomDnsTextFieldFocused)

                    if viewModel.customDnsTextField.isEmpty {
                        Text("dns.textfield.placeholder".localizedString)
                            .padding(.leading, 16)
                            .foregroundStyle(Color.Nym.textSecondary)
                            .nymTextStyle(.bodyLarge)
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
        .onChange(of: viewModel.customDnsTextField) { _, newValue in
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
            isDisabled: .constant(viewModel.isAddButtonDisabled),
            isWidthExpanded: false
        )
        .onTapGesture {
            guard !viewModel.isAddButtonDisabled else { return }
            viewModel.add()
            isCustomDnsTextFieldDirty = false
            hideKeyboard()
        }
        .accessibilityAction {
            guard !viewModel.isAddButtonDisabled else { return }
            viewModel.add()
            isCustomDnsTextFieldDirty = false
            hideKeyboard()
        }
    }

    func hideKeyboard() {
        #if os(iOS)
        KeyboardManager.shared.hideKeyboard()
        #endif
    }

    @ViewBuilder
    func dnsSaveChangesButton() -> some View {
        GenericButton(
            title: "dns.button.saveChanges".localizedString,
            isDisabled: .constant(viewModel.isSaveChangesButtonDisabled)
        )
        .padding(.horizontal, 16)
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

    @ViewBuilder
    func learnMoreLink() -> some View {
        HStack(spacing: 4) {
            Text("dns.learnMore".localizedString)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(Color.Nym.textPrimary)
                .underline()

            GenericImage(imageName: "externalLink")
                .frame(width: 12, height: 12)
                .padding(4)
                .foregroundStyle(Color.Nym.textPrimary)

            Spacer()
        }
        .onTapGesture {
            viewModel.learnMore()
        }
        .accessibilityAction {
            viewModel.learnMore()
        }
    }
}
