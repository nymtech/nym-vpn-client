import SwiftUI
import Theme
import UIComponents

#if os(iOS)
import UIKit
#endif

public struct PassphraseSignInView: View {
    @Bindable var viewModel: PassphraseSignInViewModel
    private let minHeight: CGFloat
    private let onBackTapped: () -> Void

    @Environment(\.colorScheme)
    private var colorScheme

    public init(
        viewModel: PassphraseSignInViewModel,
        minHeight: CGFloat = 0,
        onBackTapped: @escaping () -> Void
    ) {
        self.viewModel = viewModel
        self.minHeight = minHeight
        self.onBackTapped = onBackTapped
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
            Spacer(minLength: NymSpacing.large)
            VStack(spacing: NymSpacing.large) {
                heading
                textArea
                loginButton
            }
            Spacer(minLength: NymSpacing.large)
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
        .frame(maxWidth: .infinity)
        .frame(minHeight: minHeight)
        .contentShape(Rectangle())
#if os(iOS)
        .onTapGesture {
            UIApplication.shared.sendAction(
                #selector(UIResponder.resignFirstResponder),
                to: nil,
                from: nil,
                for: nil
            )
        }
#endif
    }
}

private extension PassphraseSignInView {
    var header: some View {
        ZStack {
            GenericImage(imageName: "logoText")
                .frame(width: 100, height: 27)
            HStack {
                NymBackButton(action: onBackTapped)
                Spacer()
            }
        }
    }

    var heading: some View {
        Text("passphraseSignIn.heading".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var textArea: some View {
        ZStack(alignment: .topLeading) {
            RoundedRectangle(cornerRadius: 12)
                .fill(Color.Nym.surface)
                .overlay(
                    RoundedRectangle(cornerRadius: 12)
                        .stroke(borderColor, lineWidth: 1)
                )
            ZStack(alignment: .topLeading) {
                if viewModel.passphraseText.isEmpty {
                    Text("passphraseSignIn.textArea.placeholder".localizedString)
                        .nymTextStyle(.bodyDefault)
                        .foregroundStyle(Color.Nym.textSecondary)
                        .allowsHitTesting(false)
                }
                PassphraseTextEditor(text: $viewModel.passphraseText) {
                    viewModel.loginButtonTapped()
                }
            }
            .padding(NymSpacing.large)
        }
        .frame(height: AuthLayout.passphraseTextAreaHeight)
    }

    @ViewBuilder
    var loginButton: some View {
        if viewModel.submissionState == .loading {
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.Nym.textTertiary)
                    .frame(height: 45)
                ProgressView()
                    .tint(Color.Nym.surface)
            }
        } else {
            NymButton("passphraseSignIn.loginButton".localizedString, style: .primary) {
                viewModel.loginButtonTapped()
            }
        }
    }

    var borderColor: Color {
        if viewModel.submissionState == .failed {
            return Color.Nym.error
        }
        return colorScheme == .dark
            ? Color.Nym.textPrimary.opacity(0.4)
            : Color.Nym.textPrimary.opacity(0.3)
    }
}
