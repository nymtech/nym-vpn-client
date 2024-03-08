import SwiftUI
import Modifiers
import Theme
import UIComponents

struct SurveyView: View {
    @State private var viewModel: SurveyViewModel

    init(viewModel: SurveyViewModel) {
        _viewModel = State(initialValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()

            ScrollView {
                Spacer()
                    .frame(height: 24)
                introText()
                recommendationText()
                recommendationOptions()
                provideFeedbackText()
                feedbackInputView()
                submitButton()
                Spacer()
                    .frame(height: 24)
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

private extension SurveyView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func introText() -> some View {
        Text(viewModel.introText)
            .textStyle(NymTextStyle.Label.Huge.primary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 12)
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func recommendationText() -> some View {
        Text(viewModel.recommendQuestionText)
            .textStyle(NymTextStyle.Body.Large.regular)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 12)
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func recommendationOptions() -> some View {
        VStack {
            ForEach(SurveyButtonViewModel.ButtonType.allCases) { type in
                SurveyButton(
                    viewModel:
                        SurveyButtonViewModel(
                            type: type,
                            selectedType: $viewModel.selectedRecommendation
                        )
                )
                .padding(.horizontal, 12)
                .onTapGesture {
                    viewModel.selectedRecommendation = type
                }
                Spacer()
                    .frame(height: 24)
            }
        }
    }

    @ViewBuilder
    func provideFeedbackText() -> some View {
        Text(viewModel.provideFeedbackQuestionText)
            .textStyle(NymTextStyle.Body.Large.primary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(12)
    }

    @ViewBuilder
    func feedbackInputView() -> some View {
        VStack(alignment: .leading) {
            TextField(viewModel.yourFeedbackPlacholderText, text: $viewModel.feedbackText, axis: .vertical)
                .textStyle(NymTextStyle.Body.Large.regular)
                .padding(16)
                .lineLimit(6, reservesSpace: true)

            Spacer()
        }
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
        )
        .frame(height: 156)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(Color(red: 0.29, green: 0.27, blue: 0.31), lineWidth: 1)
        }
        .padding(12)
    }

    @ViewBuilder
    func submitButton() -> some View {
        ConnectButton()
            .padding(.horizontal, 16)
            .onTapGesture {}
//        if viewModel.isSmallScreen() {
//            Spacer()
//                .frame(height: 24)
//        }
    }
}
