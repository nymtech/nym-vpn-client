import SwiftUI
import Device
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
        ScrollView {
            Spacer()
                .frame(height: 8)

            ForEach(viewModel.acknowledgements) { acknowledgement in
                AcknowledgementsRow(
                    viewModel: AcknowledgementsRowViewModel(
                        acknowledgement: acknowledgement,
                        navigationPath: viewModel.$navigationPath
                    )
                )
            }
            Spacer()
                .frame(height: 24)
        }
        .scrollIndicators(.hidden)
    }
}
