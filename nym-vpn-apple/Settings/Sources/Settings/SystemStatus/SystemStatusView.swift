import SwiftUI
import Constants
import ImpactGenerator
import ExternalLinkManager
import UIComponents
import Theme

@MainActor public struct SystemStatusView: View {
    typealias SystemStatusSection = SettingsSection<SystemStatusSectionKind>
    @EnvironmentObject private var impactGenerator: ImpactGenerator
    @EnvironmentObject private var externalLinkManger: ExternalLinkManager

    @Binding private var path: NavigationPath

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            VStack(spacing: 24) {
                SettingsList(viewModel: SettingsListViewModel(sections: sections()))
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

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension SystemStatusView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.systemStatus".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func sections() -> [SystemStatusSection] {
        [
            SystemStatusSection(
                kind: .network,
                viewModels: [
                    SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.systemStatus.networkStatus".localizedString,
                        imageName: "browse",
                        action: {
                            impactGenerator.softImpact()
                            try? externalLinkManger.openExternalURL(urlString: Constants.networkAndApiStatus.rawValue)
                        }
                    )
                ]
            ),
            SystemStatusSection(
                kind: .server,
                viewModels: [
                    SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.systemStatus.serverExplorer".localizedString,
                        imageName: "dns",
                        action: {
                            impactGenerator.softImpact()
                            try? externalLinkManger.openExternalURL(urlString: Constants.serverExplorer.rawValue)
                        }
                    ),
                    SettingsListItemViewModel(
                        accessory: .externalLink,
                        title: "settings.systemStatus.serverMonitoring".localizedString,
                        imageName: "monitoring",
                        action: {
                            impactGenerator.softImpact()
                            try? externalLinkManger.openExternalURL(urlString: Constants.serverMonitoring.rawValue)
                        }
                    )
                ]
            )
        ]
    }
}

private extension SystemStatusView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
