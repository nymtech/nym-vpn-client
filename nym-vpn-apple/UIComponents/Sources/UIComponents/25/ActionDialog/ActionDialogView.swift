import SwiftUI
import Theme

public struct ActionDialogView: View {
    @ObservedObject private var viewModel: ActionDialogViewModel
    @State private var isSpinning = false

    public init(viewModel: ActionDialogViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        ModalOverlayView(isDisplayed: $viewModel.isDisplayed) {
            VStack {
                icon()
                Spacer()
                    .frame(height: 16)
                title()
                subtitle()

                if viewModel.isLoading {
                    loadingRow()
                } else {
                    buttons()
                        .padding(24)
                }
            }
        }
        .onAppear {
            if viewModel.isLoading {
                isSpinning = true
            }
        }
        .onChange(of: viewModel.isLoading) { _, newValue in
            isSpinning = newValue
        }
    }
}

private extension ActionDialogView {
    @ViewBuilder
    func icon() -> some View {
        if let iconImageName = viewModel.configuration.systemIconImageName {
            Spacer()
                .frame(height: 24)

            Image(systemName: iconImageName)
                .foregroundStyle(viewModel.configuration.systemIconImageColor ?? Color.Nym.textPrimary)
                .frame(width: 24, height: 24)
        }
    }

    @ViewBuilder
    func title() -> some View {
        if let title = viewModel.configuration.titleLocalizedString {
            Text(title)
                .textStyle(NymTextStyle.Headline.Medium.regular)
                .foregroundStyle(Color.Nym.textPrimary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 24)

            Spacer()
                .frame(height: 16)
        }
    }

    @ViewBuilder
    func subtitle() -> some View {
        if let subtitle = viewModel.configuration.subtitleLocalizedString {
            Text(subtitle)
                .foregroundStyle(Color.Nym.textSecondary)
                .textStyle(.Body.Medium.regular)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 24)
        }
    }

    @ViewBuilder
    func buttons() -> some View {
        if viewModel.configuration.verticalButtonsLayout {
            VStack(spacing: 16) {
                combinedButtons()
            }
        } else {
            HStack(spacing: 16) {
                Spacer()
                combinedButtons()
                Spacer()
            }
        }
    }

    @ViewBuilder
    func combinedButtons() -> some View {
        if let yesLocalizedString = viewModel.configuration.yesLocalizedString {
            yesButton(text: yesLocalizedString)
        }
        if let noLocalizedString = viewModel.configuration.noLocalizedString {
            noButton(text: noLocalizedString)
        }
    }

    @ViewBuilder
    func yesButton(text: String) -> some View {
        GenericButton(title: text, style: viewModel.configuration.isYesDestructive ? .destructive : .normal)
            .onTapGesture {
#if os(iOS)
                viewModel.impactGenerator.success()
#endif
                viewModel.configuration.yesAction?()
                if viewModel.configuration.shouldCloseAfterYesAction {
                    viewModel.isDisplayed = false
                }
            }
    }

    @ViewBuilder
    func noButton(text: String) -> some View {
        GenericButton(
            title: text,
            titleColor: viewModel.configuration.isNoDestructive ? Color.Nym.error : nil,
            style: .textOnly
        )
        .onTapGesture {
#if os(iOS)
            viewModel.impactGenerator.impact()
#endif
            viewModel.configuration.noAction?()
            viewModel.isDisplayed = false
        }
    }

    @ViewBuilder
    func loadingRow() -> some View {
        if let loadingText = viewModel.displayedLoadingText {
            HStack(alignment: .center) {
                Text(loadingText)
                    .textStyle(NymTextStyle.Body.Large.regular)
                    .foregroundStyle(Color.Nym.textPrimary)
                Spacer()
                    .frame(width: 8)
                GenericImage(imageName: "activity")
                    .frame(width: 24, height: 24)
                    .rotationEffect(.degrees(isSpinning ? 360 : 0))
                    .animation(
                        .easeInOut(duration: 0.8)
                        .repeatForever(autoreverses: false),
                        value: isSpinning
                    )
            }
            .padding(24)
        }
    }
}
