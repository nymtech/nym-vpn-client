#if os(macOS)
import SwiftUI
import AppSettings
import Device
import Theme
import UIComponents

public struct AppModeView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @State private var isHovered = false
    @State private var hoveredId: Int?

    @Binding var path: NavigationPath

    public var body: some View {
        VStack {
            navbar()
            appModeOptions()
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

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension AppModeView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.appMode".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    @ViewBuilder
    func appModeOptions() -> some View {
        ForEach(AppSetting.AppMode.allCases, id: \.self) { appMode in
            SettingButton(
                viewModel:
                    SettingButtonViewModel(
                        title: appMode.localizedTitle,
                        isSelected: appSettings.appMode == appMode
                    ),
                isHovered: appMode.rawValue == hoveredId ? $isHovered : Binding.constant(false)
            )
            .onHover { newValue in
                isHovered = newValue
                hoveredId = appMode.rawValue
            }
            .onTapGesture {
                appSettings.appMode = appMode
            }
            .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
        }
    }
}

private extension AppModeView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
#endif
