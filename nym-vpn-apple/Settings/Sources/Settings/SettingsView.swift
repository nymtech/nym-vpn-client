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
    @State private var isLogoutConfirmationDisplayed = false
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
                    logoutButton()
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
        .logoutDialog(
            isPresented: $isLogoutConfirmationDisplayed,
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
        let sections = visibleSections()
        if sections.isEmpty && !viewModel.sections.isEmpty {
            sectionHeader("search.noResults".localizedString)
        } else {
            ForEach(sections, id: \.self) { section in
                sectionHeader(section.kind.title)
                    .accessibilityAddTraits(.isHeader)
                if section.kind == .account && !viewModel.isSignedIn {
                    getStartedButton()
                } else {
                    SettingsList(viewModel: SettingsListViewModel(sections: [section]))
                }
            }
        }
    }

    func getStartedButton() -> some View {
        NymButton("home.getStarted".localizedString, style: .primary) {
            viewModel.getStarted()
        }
    }

    /// Grey on purpose: the red Log out lives in Your account, never two red buttons together.
    @ViewBuilder
    func logoutButton() -> some View {
        if viewModel.isSignedIn && searchText.trimmingCharacters(in: .whitespaces).isEmpty {
            NymButton(
                "settings.logout".localizedString,
                style: .secondary,
                foregroundColor: Color.Nym.textSecondary,
                borderColor: Color.Nym.textSecondary
            ) {
                isLogoutConfirmationDisplayed = true
            }
            .padding(.top, NymSpacing.large)
        }
    }

    /// All sections, or while searching only the rows whose title matches, under their section header.
    func visibleSections() -> [SettingsSection<AppSettingsSectionKind>] {
        let getStarted = viewModel.isSignedIn ? nil : "home.getStarted".localizedString
        return SettingsSearch.filter(viewModel.sections, query: searchText) { kind in
            kind == .account ? getStarted : nil
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
