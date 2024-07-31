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
            } else if viewModel.canOpenRepository() {
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
            } else {
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
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}
