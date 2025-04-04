import SwiftUI
import Theme

public struct SettingsList: View {
    private let viewModel: SettingsListViewModel

    public init(viewModel: SettingsListViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        ForEach(viewModel.sections, id: \.self) { section in
            VStack(spacing: 0) {
                ForEach(Array(section.settingsViewModels.enumerated()), id: \.element) { index, viewModel in
                    SettingsListItem(viewModel: updatePosition(for: viewModel, with: index, section: section))
                }
            }
            Spacer()
                .frame(height: 24)
        }
        appVersionText()
            .onTapGesture(count: 3) {
                viewModel.navigateToSantasMenu()
            }
    }
}

private extension SettingsList {
    @ViewBuilder
    func appVersionText() -> some View {
        HStack {
            Text(viewModel.versionTitle)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
                .padding(EdgeInsets(top: 0, leading: 16, bottom: 16, trailing: 0))
            Spacer()
        }
    }
}

private extension SettingsList {
    func updatePosition(
        for viewModel: SettingsListItemViewModel,
        with index: Int,
        section: SettingsSection
    ) -> SettingsListItemViewModel {
        viewModel.position = SettingsListItemPosition(
            isFirst: isFirst(index: index, section: section),
            isLast: isLast(index: index, section: section)
        )
        return viewModel
    }

    func isFirst(index: Int, section: SettingsSection) -> Bool {
        index == 0
    }

    func isLast(index: Int, section: SettingsSection) -> Bool {
        index == section.settingsViewModels.count - 1
    }
}
