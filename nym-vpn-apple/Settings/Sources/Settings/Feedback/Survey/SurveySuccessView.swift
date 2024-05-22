import SwiftUI
import Modifiers
import Theme
import UIComponents

struct SurveySuccessView: View {
    private var viewModel: SurveySuccessViewModel

    init(viewModel: SurveySuccessViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        VStack {
            navbar()
            successImage()
            successCopyText()
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

private extension SurveySuccessView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func successImage() -> some View {
        SuccessImage()
            .padding(24)
    }

    @ViewBuilder
    func successCopyText() -> some View {
        Text(viewModel.copyText)
            .textStyle(.Body.Large.primary)
            .foregroundStyle(NymColor.surveyText)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }
}
