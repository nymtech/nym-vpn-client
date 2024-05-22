import SwiftUI
import Modifiers
import Theme
import UIComponents

struct LegalView: View {
    @State private var viewModel: LegalViewModel

    init(viewModel: LegalViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()
            Spacer()
                .frame(height: 24)
            section()
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

private extension LegalView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func section() -> some View {
        VStack(spacing: 0) {
            ForEach(viewModel.viewModels, id: \.self) { viewModel in
                SettingsListItem(viewModel: viewModel)
            }
        }
    }
}
