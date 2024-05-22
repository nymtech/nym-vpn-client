import SwiftUI
import AppSettings
#if os(iOS)
import KeyboardManager
#endif
import Modifiers
import Theme
import UIComponents

struct SurveyView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @StateObject private var viewModel: SurveyViewModel
    @FocusState private var isFocused: Bool

    init(viewModel: SurveyViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    var body: some View {
        VStack {
            navbar()
#if os(iOS)
            KeyboardHostView {
                scrollViewContent()
            }
#elseif os(macOS)
            scrollViewContent()
#endif
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
    func scrollViewContent() -> some View {
        ScrollView {
            Spacer()
                .frame(height: 24)
            introText()
            recommendationText()
            recommendationOptions()
            provideFeedbackText()
            feedbackInputView()
            if viewModel.error != .noError, let errorMessageTitle = viewModel.error.localizedTitle {
                errorMessageView(title: errorMessageTitle)
            }
            submitButton()
            Spacer()
                .frame(height: 24)
        }
        .onTapGesture {
            isFocused = false
        }
    }

    @ViewBuilder
    func introText() -> some View {
        Text(viewModel.introText)
            .textStyle(NymTextStyle.Label.Huge.primary)
            .foregroundStyle(NymColor.surveyText)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, 12)
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func recommendationText() -> some View {
        Text(viewModel.recommendQuestionText)
            .textStyle(NymTextStyle.Body.Large.regular)
            .foregroundStyle(NymColor.surveyText)
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
                            hasError: viewModel.surveyButtonShouldShowError,
                            selectedType: $viewModel.selectedRecommendation
                        )
                )
                .padding(.horizontal, 12)
                .onTapGesture {
                    isFocused = false
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
            .foregroundStyle(NymColor.surveyText)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(12)
    }

    @ViewBuilder
    func feedbackInputView() -> some View {
        VStack(alignment: .leading) {
            TextField(viewModel.yourFeedbackPlacholderText, text: $viewModel.feedbackText, axis: .vertical)
                .textStyle(NymTextStyle.Body.Large.regular)
                .textFieldStyle(PlainTextFieldStyle())
                .padding(16)
                .lineLimit(6, reservesSpace: true)
            Spacer()
        }
        .focused($isFocused)
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
        )
        .frame(height: 156)
        .cornerRadius(8)
        .overlay {
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(viewModel.textFieldStrokeColor, lineWidth: 1)
        }
        .padding(EdgeInsets(top: 12, leading: 12, bottom: viewModel.error != .noError ? 4 : 12, trailing: 12))
    }

    @ViewBuilder
    func errorMessageView(title: String) -> some View {
        HStack {
            Text(title)
                .foregroundStyle(NymColor.sysError)
                .textStyle(NymTextStyle.Body.Small.primary)
            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 28, bottom: 16, trailing: 28))
    }

    @ViewBuilder
    func submitButton() -> some View {
        GenericButton(title: viewModel.submitButtonTitle)
            .padding(.horizontal, 16)
            .onTapGesture {
                viewModel.submit()
            }
        if appSettings.isSmallScreen {
            Spacer()
                .frame(height: 24)
        }
    }
}
