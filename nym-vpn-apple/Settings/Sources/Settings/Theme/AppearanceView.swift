import SwiftUI
import AppSettings
import Device
import Theme
import UIComponents

public struct AppearanceView: View {
    @ObservedObject private var viewModel: AppearanceViewModel
    @State private var isHovered = false
    @State private var hoveredId: Int?

    public init(viewModel: AppearanceViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
            themeOptions()
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
                    ),
                isHovered: appearance.rawValue == hoveredId ? $isHovered : Binding.constant(false)
            )
            .onHover { newValue in
                isHovered = newValue
                hoveredId = appearance.rawValue
            }
            .onTapGesture {
                viewModel.updateAppearance(with: appearance)
            }
            .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
        }
    }
}
