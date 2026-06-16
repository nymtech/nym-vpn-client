#if os(iOS)
import SwiftUI
import Theme
import UIComponents

struct AppIconView: View {
    @ObservedObject private var viewModel: AppIconViewModel

    private let columns = [GridItem(.adaptive(minimum: 96), spacing: 24)]

    init(viewModel: AppIconViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack(spacing: 0) {
            navbar()
            ScrollView {
                LazyVGrid(columns: columns, spacing: 24) {
                    ForEach(viewModel.icons, id: \.self) { icon in
                        iconCell(icon)
                    }
                }
                .frame(maxWidth: MagicNumbers.maxWidth)
                .padding(24)
            }
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
    }
}

private extension AppIconView {
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    func iconCell(_ icon: AppIcon) -> some View {
        let isSelected = viewModel.current == icon
        return VStack(spacing: 8) {
            Image(icon.previewImageName, bundle: .module)
                .resizable()
                .scaledToFit()
                .frame(width: 80, height: 80)
                .clipShape(RoundedRectangle(cornerRadius: 18, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 18, style: .continuous)
                        .stroke(isSelected ? Color.Nym.primary : Color.clear, lineWidth: 3)
                )
            Text(icon.title)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textPrimary)
        }
        .contentShape(Rectangle())
        .onTapGesture {
            Task { await viewModel.select(icon) }
        }
        .accessibilityElement(children: .combine)
        .accessibilityLabel(icon.title)
        .accessibilityAddTraits(isSelected ? [.isButton, .isSelected] : [.isButton])
    }
}
#endif
