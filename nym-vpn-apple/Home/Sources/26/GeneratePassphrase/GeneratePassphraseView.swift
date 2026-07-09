import SwiftUI
import AccountPrefetchGates
import CredentialsManager
import Theme
import UIComponents

public struct GeneratePassphraseView: View {
    @Bindable var viewModel: GeneratePassphraseViewModel
    private let minHeight: CGFloat
    private let onBackTapped: () -> Void

    public init(
        viewModel: GeneratePassphraseViewModel,
        minHeight: CGFloat = 0,
        onBackTapped: @escaping () -> Void
    ) {
        self.viewModel = viewModel
        self.minHeight = minHeight
        self.onBackTapped = onBackTapped
    }

    public var body: some View {
        VStack(spacing: AuthLayout.processingCarouselStackSpacing) {
            header
            stepIndicator
            WaveDotsView()
                .padding(.top, AuthLayout.carouselLoaderTopSpacing)
                .padding(.bottom, AuthLayout.carouselLoaderBottomSpacing)
            switchingTitles
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.processingCarouselVerticalPadding)
        .frame(maxWidth: .infinity)
        .frame(height: minHeight > 0 ? minHeight : nil, alignment: .top)
        .task {
            viewModel.start()
        }
        .alert(
            viewModel.errorMessage ?? "",
            isPresented: Binding(
                get: { viewModel.errorMessage != nil },
                set: { if !$0 { viewModel.dismissError() } }
            )
        ) {
            Button("ok".localizedString, role: .cancel) {
                viewModel.dismissError()
            }
            Button("retry".localizedString) {
                viewModel.retry()
            }
        }
    }
}

private extension GeneratePassphraseView {
    var header: some View {
        AuthDrawerHeader(onBackTapped: onBackTapped)
    }

    var stepIndicator: some View {
        StepView(
            stepCount: 4,
            currentStep: Binding(
                get: { viewModel.currentStep },
                set: { _ in }
            )
        )
    }

    var switchingTitles: some View {
        let title = LoginProcessingUI.settingUpTitleKey.localizedString
        return SwitchingTitlesView(
            pairs: LoginProcessingUI.carouselStepRange.map { _ in (title, "") },
            didFinishAnimating: Binding(
                get: { viewModel.didFinishAnimatingText },
                set: { newValue in
                    if newValue { viewModel.animationDidFinish() }
                }
            ),
            timerDidTick: {
                viewModel.animationDidAdvance()
            }
        )
    }
}
