import SwiftUI
import AcknowList
import UIComponents
import Theme

public struct AcknowledgementsRow: View {
    private let viewModel: AcknowledgementsRowViewModel

    public init(viewModel: AcknowledgementsRowViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            if viewModel.textExistsAndCanFetchLicence {
                settingsListItemNavigateToLicence()
            } else if viewModel.canOpenRepository() {
                settingsListItemOpenExternalBrowser()
            } else {
                settingsListItemOpenExternalBrowserNoArrow()
            }
        }
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

extension AcknowledgementsRow {
    @ViewBuilder
    func settingsListItemNavigateToLicence() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: viewModel.acknowledgement.title,
                position: .init(isFirst: true, isLast: true),
                action: {
                    viewModel.navigateToLicence()
                }
            )
        )
    }

    @ViewBuilder
    func settingsListItemOpenExternalBrowser() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .arrow,
                title: viewModel.acknowledgement.title,
                position: .init(isFirst: true, isLast: true),
                action: {
                    viewModel.openExternalBrowser()
                }
            )
        )
    }

    @ViewBuilder
    func settingsListItemOpenExternalBrowserNoArrow() -> some View {
        SettingsListItem(
            viewModel: SettingsListItemViewModel(
                accessory: .empty,
                title: viewModel.acknowledgement.title,
                position: .init(isFirst: true, isLast: true),
                action: {
                    viewModel.openExternalBrowser()
                }
            )
        )
    }
}
