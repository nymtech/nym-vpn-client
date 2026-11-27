import SwiftUI
import AppSettings
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import Device
import ConfigurationManager
import UIComponents
import Theme

public struct SettingsView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @EnvironmentObject private var connectionManager: ConnectionManager
    @StateObject private var viewModel: SettingsViewModel
    @State private var searchText = ""
    @FocusState private var isSearchFocused: Bool
#if os(macOS)
    @State private var autologinState = AutologinState()
#endif

    public init(viewModel: SettingsViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        SettingsFlowCoordinator(flowState: viewModel, content: content)
    }
}

private extension SettingsView {
    @ViewBuilder
    func content() -> some View {
        VStack(spacing: 0) {
            navbar()
            ScrollView {
                VStack(spacing: 0) {
                    Spacer()
                        .frame(height: NymSpacing.section)
                    SearchView(
                        searchText: $searchText,
                        isSearchFocused: $isSearchFocused,
                        placeholder: "settings.search.placeholder".localizedString
                    )
                    Spacer()
                        .frame(height: NymSpacing.section)
                    settingsList()
                }
                .padding(.horizontal, NymSpacing.large)
            }
            .scrollIndicators(.never)
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
#if os(macOS)
        .autologinOverlay(
            state: autologinState,
            onRetry: { autologinState.start(kind: .autologinRenew, using: credentialsManager) }
        )
#endif
#if os(iOS)
        .fullScreenCover(isPresented: $viewModel.isCodeEntryPresented) {
            FreepassOrPairingEntryView(startsWithScanner: true) { outcome in
                viewModel.handleCodeEntry(outcome)
            }
            .interactiveDismissDisabled()
        }
#elseif os(macOS)
        .sheet(isPresented: $viewModel.isCodeEntryPresented) {
            FreepassCodeEntryView { outcome in
                viewModel.handleCodeEntry(outcome)
            }
        }
#endif
        .logoutDialog(
            isPresented: $viewModel.isLogoutConfirmationDisplayed,
            credentialsManager: credentialsManager,
            connectionManager: connectionManager,
            onLoggedOut: { viewModel.navigateToRoot() }
        )
        .onAppear {
#if os(macOS)
            viewModel.autologinState = autologinState
#endif
            Task {
                await credentialsManager.updateAccountSummary()
            }
        }
        .onDisappear {
            // a pushed subpage hides this screen; start unfiltered when coming back
            searchText = ""
        }
    }

    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.settingsTitle,
            backgroundColorOverride: Color.Nym.background,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() }),
            rightButton: CustomNavBarButton(type: .close, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func settingsList() -> some View {
        let sections = SettingsSearch.filter(viewModel.sections, query: searchText) { viewModel.blocks(for: $0) }
        if sections.isEmpty && !viewModel.sections.isEmpty {
            sectionHeader("search.noResults".localizedString)
        } else {
            ForEach(sections, id: \.self) { section in
                if let title = section.kind.title {
                    sectionHeader(title)
                        .accessibilityAddTraits(.isHeader)
                }
                ForEach(section.blocks.filter(\.isAboveRows), id: \.self) { block($0) }
                if !section.rows.isEmpty {
                    SettingsList(
                        viewModel: SettingsListViewModel(
                            sections: [SettingsSection(kind: section.kind, viewModels: section.rows)]
                        )
                    )
                }
                ForEach(section.blocks.filter { !$0.isAboveRows }, id: \.self) { block($0) }
            }
        }
    }

    @ViewBuilder
    func block(_ block: SettingsBlock) -> some View {
        switch block {
        case .getStarted:
            NymButton(block.title, style: .primary) {
                viewModel.getStarted()
            }
            .padding(.bottom, NymSpacing.large)
        case .appVersion:
            appVersionText(block.title)
                .padding(.top, NymSpacing.section)
                .onTapGesture(count: 3) {
                    viewModel.navigateToSantasMenu()
                }
        }
    }

    func sectionHeader(_ title: String) -> some View {
        HStack {
            Text(title)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodySmall)
                .padding(.bottom, NymSpacing.small)
            Spacer()
        }
    }

    func appVersionText(_ title: String) -> some View {
        VStack(spacing: 0) {
            HStack {
                Text(title)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
                    .padding(.bottom, NymSpacing.large)
                Spacer()
            }
            Spacer()
                .frame(height: NymSpacing.section)
        }
    }
}
