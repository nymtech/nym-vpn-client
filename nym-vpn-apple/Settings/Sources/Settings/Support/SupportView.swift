import SwiftUI
import Modifiers
import Theme
import UIComponents

struct SupportView: View {
    @State private var viewModel: SupportViewModel

    init(viewModel: SupportViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()
            Spacer()
                .frame(height: 24)
            sections()
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

private extension SupportView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func sections() -> some View {
        ForEach(viewModel.sections, id: \.self) { viewModel in
            SettingsListItem(viewModel: viewModel)
            Spacer()
                .frame(height: 24)
        }
    }
}
