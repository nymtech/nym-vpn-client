import SwiftUI
import ExternalLinkManager
import Theme
import UIComponents

public struct AppearanceView: View {
    let externalLinkManager: ExternalLinkManager = .shared

    @Binding var path: NavigationPath
#if os(macOS)
    @StateObject private var launchAtStartupManager = LaunchAtStartupManager()
#endif

    public init(path: Binding<NavigationPath>) {
        _path = path
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()

            VStack(spacing: 0) {
                Spacer()
                    .frame(height: 24)
                language()
                    .frame(maxWidth: MagicNumbers.maxWidth)
                Spacer()
                    .frame(height: 24)
                theme()
                    .frame(maxWidth: MagicNumbers.maxWidth)
#if os(macOS)
                Spacer()
                    .frame(height: 24)
                appMode()
                    .frame(maxWidth: MagicNumbers.maxWidth)
                Spacer()
                    .frame(height: 24)
                launchAtStartup()
                    .frame(maxWidth: MagicNumbers.maxWidth)
#endif
                Spacer()
            }
            .padding(.horizontal, 16)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
    }
}

extension AppearanceView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.appearance".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func language() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.language".localizedString,
                imageName: "language",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToLanguage()
                }
            )
        )
    }

    func theme() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.displayTheme".localizedString,
                imageName: "displayTheme",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToDisplayTheme()
                }
            )
        )
    }
}

#if os(macOS)
private extension AppearanceView {
    func appMode() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.appMode".localizedString,
                systemImageName: "menubar.dock.rectangle",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    Task { @MainActor in
                        navigateToAppMode()
                    }
                }
            )
        )
    }

    @MainActor func navigateToAppMode() {
        path.append(SettingLink.appMode)
    }

    func launchAtStartup() -> some View {
        let toggleBinding = Binding<Bool>(
            get: { launchAtStartupManager.isEnabled },
            set: { newValue in
                launchAtStartupManager.isEnabled = newValue
                launchAtStartupManager.apply(newValue)
            }
        )
        return SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .toggle(isOn: toggleBinding),
                title: "settings.launchAtStartup".localizedString,
                imageName: "launchAtStartup",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {}
            )
        )
        .id(launchAtStartupManager.isEnabled)
    }
}
#endif
