import SwiftUI
import ConnectionTypes
import CredentialsManager
import ExternalLinkManager
import ImpactGenerator
import Theme
import UIComponents

public struct SignInView: View {
    private let credentialsManager: CredentialsManager
    private let rootMinHeight: CGFloat
    private let onBackTapped: () -> Void

    @State private var passphraseViewModel: PassphraseSignInViewModel
    @State private var showsPassphrase = false
    @State private var cardHeight: CGFloat?
    @State private var isPrivyLoading = false
    @State private var privyAlertMessage: String?
    @State private var privyTask: Task<Void, Never>?

    public init(
        credentialsManager: CredentialsManager,
        rootMinHeight: CGFloat = 0,
        onBackTapped: @escaping () -> Void
    ) {
        self.credentialsManager = credentialsManager
        self.rootMinHeight = rootMinHeight
        self.onBackTapped = onBackTapped
        _passphraseViewModel = State(
            wrappedValue: PassphraseSignInViewModel(credentialsManager: credentialsManager)
        )
    }

    public var body: some View {
        ZStack(alignment: .top) {
            if !showsPassphrase {
                SignInRootContent(
                    minHeight: rootMinHeight,
                    isPrivyLoading: isPrivyLoading,
                    onBackTapped: onBackTapped,
                    onPassphraseTapped: { showsPassphrase = true },
                    onSocialsTapped: startPrivyLogin
                )
                .fixedSize(horizontal: false, vertical: true)
                .trackHeight { cardHeight = $0 }
                .transition(.slideFade(from: .leading))
            } else {
                PassphraseSignInView(
                    viewModel: passphraseViewModel,
                    minHeight: rootMinHeight,
                    onBackTapped: { showsPassphrase = false }
                )
                .fixedSize(horizontal: false, vertical: true)
                .trackHeight { cardHeight = $0 }
                .transition(.slideFade(from: .trailing))
            }
        }
        .frame(maxWidth: .infinity)
        .frame(height: cardHeight)
        .animation(.easeInOut, value: showsPassphrase)
        .clipped()
        .alert(
            privyAlertMessage ?? "",
            isPresented: Binding(
                get: { privyAlertMessage != nil },
                set: { if !$0 { privyAlertMessage = nil } }
            )
        ) {
            Button("ok".localizedString, role: .cancel) {}
        }
    }

    private func startPrivyLogin() {
        guard !isPrivyLoading else { return }
        isPrivyLoading = true
        privyTask?.cancel()
        privyTask = Task { @MainActor in
            defer { isPrivyLoading = false }
            do {
                let url = try await credentialsManager.privyLogin(kind: .privy)
#if os(iOS)
                try ExternalLinkManager.shared.openInAppBrowser(urlString: url)
#elseif os(macOS)
                try ExternalLinkManager.shared.openExternalURL(urlString: url)
#endif
            } catch is CancellationError {
                return
            } catch {
                privyAlertMessage = error.localizedDescription
            }
        }
    }
}

struct SignInRootContent: View {
    var minHeight: CGFloat = 0
    let isPrivyLoading: Bool
    let onBackTapped: () -> Void
    let onPassphraseTapped: () -> Void
    let onSocialsTapped: () -> Void

    var body: some View {
        VStack(spacing: 0) {
            header
            Spacer(minLength: NymSpacing.large)
            VStack(spacing: AuthLayout.stackSpacing) {
                heading
                buttons
            }
            Spacer(minLength: NymSpacing.large)
            socialsFootnote
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
        .frame(maxWidth: .infinity)
        .frame(minHeight: minHeight)
    }
}

private extension SignInRootContent {
    var header: some View {
        ZStack {
            GenericImage(imageName: "logoText")
                .frame(width: 100, height: 27)
            HStack {
                NymBackButton {
                    ImpactGenerator.shared.softImpact()
                    onBackTapped()
                }
                Spacer()
            }
        }
    }

    var heading: some View {
        Text("signIn.heading".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var buttons: some View {
        VStack(spacing: NymSpacing.component) {
            NymButton(
                "signIn.loginWith24Words".localizedString,
                style: .primary,
                isDisabled: isPrivyLoading
            ) {
                ImpactGenerator.shared.softImpact()
                onPassphraseTapped()
            }
            socialsButton
        }
    }

    @ViewBuilder
    var socialsButton: some View {
        if isPrivyLoading {
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.Nym.gray1)
                    .frame(height: 45)
                ProgressView()
                    .tint(Color.Nym.gray12)
            }
        } else {
            NymButton(
                "signIn.loginUsingSocials".localizedString,
                style: .secondary,
                foregroundColor: .Nym.textPrimary,
                borderColor: .Nym.textPrimary,
                trailingSystemImage: "arrow.up.forward.square"
            ) {
                ImpactGenerator.shared.softImpact()
                onSocialsTapped()
            }
        }
    }

    var socialsFootnote: some View {
        Text("signIn.socialsFootnote".localizedString)
            .nymTextStyle(.bodySmall)
            .foregroundStyle(Color.Nym.textSecondary)
            .multilineTextAlignment(.center)
    }
}
