import SwiftUI
import Device
import Theme
import UIComponents

struct FeedbackView: View {
    @State private var viewModel: FeedbackViewModel

    init(viewModel: FeedbackViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()
            Spacer()
                .frame(height: 24)
            sections()
                .frame(maxWidth: Device.type == .ipad ? 358 : .infinity)
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

private extension FeedbackView {
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
