import SwiftUI
import Theme

public struct SettingsList<Kind: SettingsSectionKind>: View {
    private let viewModel: SettingsListViewModel<Kind>

    public init(viewModel: SettingsListViewModel<Kind>) {
        self.viewModel = viewModel
    }

    public var body: some View {
        ForEach(viewModel.sections, id: \.self) { section in
            VStack(spacing: 0) {
                ForEach(Array(section.viewModels.enumerated()), id: \.element) { index, viewModel in
                    SettingsListItem(
                        viewModel: updatePosition(for: viewModel, with: index, section: section)
                    )
                }
            }
            .padding(.bottom, 16)
        }
    }
}

private extension SettingsList {
    func updatePosition(
        for viewModel: SettingsListItemViewModel,
        with index: Int,
        section: SettingsSection<Kind>
    ) -> SettingsListItemViewModel {
        viewModel.position = SettingsListItemPosition(
            isFirst: isFirst(index: index, section: section),
            isLast: isLast(index: index, section: section)
        )
        return viewModel
    }

    func isFirst(index: Int, section: SettingsSection<Kind>) -> Bool {
        index == 0
    }

    func isLast(index: Int, section: SettingsSection<Kind>) -> Bool {
        index == section.viewModels.count - 1
    }
}
