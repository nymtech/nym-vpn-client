import SwiftUI
import Modifiers
import Theme
import UIComponents

struct AcknowledgmentsView: View {
        @ObservedObject private var viewModel: AcknowledgeMentsViewModel

        init(viewModel: AcknowledgeMentsViewModel) {
            self.viewModel = viewModel
        }

    var body: some View {
        VStack(spacing: 0) {
            navbar()
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

private extension AcknowledgmentsView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func section() -> some View {
        if let acknowledgements = viewModel.acknoledgementsList?.acknowledgements {
            ScrollView {
                Spacer()
                    .frame(height: 8)

                ForEach(acknowledgements) { acknowledgement in
                    AcknowledgementsRow(
                        viewModel: AcknowledgementsRowViewModel(
                            acknowledgement: acknowledgement,
                            path: viewModel.$path
                        )
                    )
                }
                Spacer()
                    .frame(height: 24)
            }
        }
    }
}
