#if os(iOS)
import SwiftUI
import AppSettings
import Theme
import UIComponents

public struct AppIconView: View {
    @ObservedObject private var viewModel: AppIconViewModel

    public init(viewModel: AppIconViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            iconGrid()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.surfaceBg
                .ignoresSafeArea()
        }
        .alert(
            "settings.appIcon.confirmTitle".localizedString,
            isPresented: Binding(
                get: { viewModel.pendingIcon != nil },
                set: { if !$0 { viewModel.cancelChange() } }
            )
        ) {
            Button("settings.appIcon.confirmAction".localizedString) {
                Task { await viewModel.confirmChange() }
            }
            Button("settings.appIcon.cancel".localizedString, role: .cancel) {
                viewModel.cancelChange()
            }
        } message: {
            Text("settings.appIcon.confirmBody".localizedString)
        }
    }
}

private extension AppIconView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func iconGrid() -> some View {
        let columns = [GridItem(.flexible()), GridItem(.flexible())]
        LazyVGrid(columns: columns, spacing: 16) {
            ForEach(viewModel.icons) { icon in
                iconCard(for: icon)
            }
        }
        .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
    }

    @ViewBuilder
    func iconCard(for icon: AppIcon) -> some View {
        let isSelected = viewModel.selectedIcon == icon
        VStack(spacing: 8) {
            Image(icon.previewAssetName)
                .resizable()
                .scaledToFit()
                .frame(width: 80, height: 80)
                .cornerRadius(18)
                .overlay(
                    RoundedRectangle(cornerRadius: 18)
                        .inset(by: 0.5)
                        .stroke(
                            isSelected ? Color.Nym.brandPrimary : Color.clear,
                            lineWidth: 2
                        )
                )
            Text(icon.localizedTitleKey.localizedString)
                .foregroundStyle(isSelected ? Color.Nym.brandPrimary : Color.Nym.textPrimary)
                .textStyle(.Body.Medium.regular)
        }
        .frame(maxWidth: .infinity)
        .padding(12)
        .background(Color.Nym.surfaceElev)
        .cornerRadius(12)
        .overlay(
            RoundedRectangle(cornerRadius: 12)
                .inset(by: 0.5)
                .stroke(
                    isSelected ? Color.Nym.brandPrimary : Color.Nym.surfaceHair,
                    lineWidth: 1
                )
        )
        .onTapGesture {
            viewModel.requestChange(to: icon)
        }
    }
}
#endif
