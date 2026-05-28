import SwiftUI
import ExternalLinkManager
import Theme
import UIComponents

public struct AppearanceView: View {
    let externalLinkManager: ExternalLinkManager = .shared

    @Binding var path: NavigationPath

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
#if os(iOS)
                Spacer()
                    .frame(height: 24)
                appIcon()
                    .frame(maxWidth: MagicNumbers.maxWidth)
#endif
#if os(macOS)
                Spacer()
                    .frame(height: 24)
                appMode()
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

#if os(iOS)
    func appIcon() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: "settings.appIcon.title".localizedString,
                systemImageName: "square.grid.2x2",
                position: SettingsListItemPosition(isFirst: true, isLast: true),
                action: {
                    navigateToAppIcon()
                }
            )
        )
    }
#endif
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
}
#endif
