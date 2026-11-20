import SwiftUI
import AppSettings
import ConnectionTypes
import CredentialsManager
import Device
import ConfigurationManager
import UIComponents
import Theme

public struct SettingsView: View {
    @EnvironmentObject private var credentialsManager: CredentialsManager
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
                    Spacer()
                        .frame(height: NymSpacing.section)
                    appVersionText()
                        .onTapGesture(count: 3) {
                            viewModel.navigateToSantasMenu()
                        }
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
            CodeEntryView(startsWithScanner: true) { outcome in
                viewModel.handleCodeEntry(outcome)
            }
            .interactiveDismissDisabled()
        }
#endif
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
        let sections = visibleSections()
        if sections.isEmpty && !viewModel.sections.isEmpty {
            sectionHeader("search.noResults".localizedString)
        } else {
            ForEach(sections, id: \.self) { section in
                sectionHeader(section.kind.title)
                    .accessibilityAddTraits(.isHeader)
                SettingsList(viewModel: SettingsListViewModel(sections: [section]))
            }
        }
    }

    /// All sections, or while searching only the rows whose title matches, under their section header.
    func visibleSections() -> [SettingsSection<AppSettingsSectionKind>] {
        let query = searchText.trimmingCharacters(in: .whitespaces)
        guard !query.isEmpty else { return viewModel.sections }
        return viewModel.sections.compactMap { section in
            let hits = section.viewModels.filter { $0.title.localizedCaseInsensitiveContains(query) }
            return hits.isEmpty ? nil : SettingsSection(kind: section.kind, viewModels: hits)
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

    func appVersionText() -> some View {
        VStack(spacing: 0) {
            HStack {
                Text(viewModel.versionTitle)
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
