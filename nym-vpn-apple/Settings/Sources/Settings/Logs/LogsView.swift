import SwiftUI
import Theme
import UIComponents

public struct LogsView: View {
    private let viewModel: LogsViewModel

    public init(viewModel: LogsViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
            Spacer()

            ScrollView {
                Text(viewModel.logs)
                    .padding()
            }

            GenericButton(title: viewModel.copyLocalizedString)
                .padding(16)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension LogsView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }
}
