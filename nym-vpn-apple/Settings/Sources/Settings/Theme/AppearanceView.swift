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
            Spacer()
                .frame(height: 24)
            language()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
                .frame(height: 24)
            theme()
                .frame(maxWidth: MagicNumbers.maxWidth)
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
}

extension AppearanceView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.appearance".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    @ViewBuilder
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

    @ViewBuilder
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
