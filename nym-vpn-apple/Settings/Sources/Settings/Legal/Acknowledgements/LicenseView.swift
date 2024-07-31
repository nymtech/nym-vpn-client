import SwiftUI
import Theme
import UIComponents

public struct LicenseView: View {
    @ObservedObject var viewModel: LicenseViewModel

    public init(viewModel: LicenseViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
                .padding(0)

            ScrollView {
                Text(viewModel.acknowledgement.title)
                    .font(.title)
                    .padding()
                Text(viewModel.acknowledgement.text ?? "")
                    .font(.body)
                    .padding()
            }
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

private extension LicenseView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }
}
