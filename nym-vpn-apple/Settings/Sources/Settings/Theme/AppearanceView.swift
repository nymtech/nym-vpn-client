import SwiftUI
import Modifiers
import Theme
import UIComponents

public struct AppearanceView: View {
    @ObservedObject private var viewModel: AppearanceViewModel

    public init(viewModel: AppearanceViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
            themeOptions()
            Spacer()
        }
        .appearanceUpdate()
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension AppearanceView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func themeOptions() -> some View {
        ForEach(viewModel.themes, id: \.self) { appearance in
            SettingButton(
                viewModel:
                    SettingButtonViewModel(
                        title: viewModel.appearanceTitle(for: appearance),
                        subtitle: viewModel.appearanceSubtitle(for: appearance),
                        isSelected: viewModel.currentAppearance == appearance
                    )
            )
            .onTapGesture {
                viewModel.updateAppearance(with: appearance)
            }
            .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
        }
    }
}
