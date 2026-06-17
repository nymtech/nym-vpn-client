import AuthenticationServices
import SwiftUI
import AccountPrefetchGates
import ConnectionTypes
import CredentialsManager
import ExternalLinkManager
import ImpactGenerator
import Theme
import UIComponents

public struct SignUpView: View {
    enum PrivyTarget: Equatable {
        case anonymous
        case social
    }

#if os(iOS)
    enum Step: Equatable {
        case root
        case generate
    }
#endif

    private let credentialsManager: CredentialsManager
    private let rootMinHeight: CGFloat
    private let onBackTapped: () -> Void
    private let onWillRegister: () -> Void
    private let onPrivyAuthWillBegin: () -> Void
    private let onAuthHandoffCancelled: () -> Void
    private let onAuthCompleted: (AuthCompletionOutcome) -> Void

#if os(iOS)
    @State private var generateViewModel: GeneratePassphraseViewModel
    @State private var step: Step = .root
#endif
    @State private var cardHeight: CGFloat?
    @State private var privyLoadingTarget: PrivyTarget?
    @State private var privyAlertMessage: String?
    @State private var privyTask: Task<Void, Never>?

    public init(
        credentialsManager: CredentialsManager,
        rootMinHeight: CGFloat = 0,
        onBackTapped: @escaping () -> Void,
        onWillRegister: @escaping () -> Void = {},
        onPrivyAuthWillBegin: @escaping () -> Void = {},
        onAuthHandoffCancelled: @escaping () -> Void = {},
        onAuthCompleted: @escaping (AuthCompletionOutcome) -> Void = { _ in }
    ) {
        self.credentialsManager = credentialsManager
        self.rootMinHeight = rootMinHeight
        self.onBackTapped = onBackTapped
        self.onWillRegister = onWillRegister
        self.onPrivyAuthWillBegin = onPrivyAuthWillBegin
        self.onAuthHandoffCancelled = onAuthHandoffCancelled
        self.onAuthCompleted = onAuthCompleted
#if os(iOS)
        let viewModel = GeneratePassphraseViewModel(credentialsManager: credentialsManager)
        viewModel.onWillRegister = onWillRegister
        viewModel.onAuthHandoffCancelled = onAuthHandoffCancelled
        viewModel.onAuthCompleted = onAuthCompleted
        _generateViewModel = State(wrappedValue: viewModel)
#endif
    }

    public var body: some View {
        ZStack(alignment: .top) {
#if os(iOS)
            switch step {
            case .root:
                rootContent
            case .generate:
                GeneratePassphraseView(
                    viewModel: generateViewModel,
                    minHeight: rootMinHeight,
                    onBackTapped: { step = .root }
                )
                .fixedSize(horizontal: false, vertical: true)
                .trackHeight { cardHeight = $0 }
                .transition(.slideFade(from: .trailing))
            }
#else
            rootContent
#endif
        }
        .frame(maxWidth: .infinity)
        .frame(height: cardHeight)
#if os(iOS)
        .animation(.easeInOut, value: step)
#endif
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

    private var rootContent: some View {
        SignUpRootContent(
            minHeight: rootMinHeight,
            privyLoadingTarget: privyLoadingTarget,
            onBackTapped: onBackTapped,
            onAnonymousTapped: anonymousAccountTapped,
            onSocialsTapped: { startPrivyLogin(target: .social) }
        )
        .fixedSize(horizontal: false, vertical: true)
        .trackHeight { cardHeight = $0 }
        .transition(.slideFade(from: .leading))
    }

    private func anonymousAccountTapped() {
#if os(iOS)
        step = .generate
#elseif os(macOS)
        startPrivyLogin(target: .anonymous)
#endif
    }

    private func startPrivyLogin(target: PrivyTarget) {
        guard privyLoadingTarget == nil else { return }
        privyLoadingTarget = target
        onPrivyAuthWillBegin()
        privyTask?.cancel()
        privyTask = Task { @MainActor in
            defer { privyLoadingTarget = nil }
            do {
                let kind: NymDeeplinkKind = (target == .anonymous) ? .createAccount : .privy
                let url = try await credentialsManager.privyLogin(kind: kind)
                try await ExternalLinkManager.shared.presentPrivyAuthSession(urlString: url)
            } catch is CancellationError {
                onAuthHandoffCancelled()
                return
            } catch let error as ASWebAuthenticationSessionError where error.code == .canceledLogin {
                onAuthHandoffCancelled()
                return
            } catch {
                onAuthHandoffCancelled()
                privyAlertMessage = error.localizedDescription
            }
        }
    }
}

struct SignUpRootContent: View {
    var minHeight: CGFloat = 0
    let privyLoadingTarget: SignUpView.PrivyTarget?
    let onBackTapped: () -> Void
    let onAnonymousTapped: () -> Void
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

private extension SignUpRootContent {
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
        Text("signUp.heading".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)
            .multilineTextAlignment(.center)
    }

    var buttons: some View {
        VStack(spacing: NymSpacing.component) {
            anonymousButton
            socialsButton
        }
    }

    @ViewBuilder
    var anonymousButton: some View {
        if privyLoadingTarget == .anonymous {
            loadingButton
        } else {
            NymButton(
                "signUp.anonymousAccount".localizedString,
                style: .primary,
                isDisabled: privyLoadingTarget != nil
            ) {
                ImpactGenerator.shared.softImpact()
                onAnonymousTapped()
            }
        }
    }

    @ViewBuilder
    var socialsButton: some View {
        if privyLoadingTarget == .social {
            loadingButton
        } else {
            NymButton(
                "signUp.signUpWithSocials".localizedString,
                style: .secondary,
                foregroundColor: .Nym.textPrimary,
                borderColor: .Nym.textPrimary,
                trailingSystemImage: "arrow.up.forward.square",
                isDisabled: privyLoadingTarget != nil
            ) {
                ImpactGenerator.shared.softImpact()
                onSocialsTapped()
            }
        }
    }

    var loadingButton: some View {
        ZStack {
            RoundedRectangle(cornerRadius: 8)
                .fill(Color.Nym.textTertiary)
                .frame(height: 45)
            ProgressView()
                .tint(Color.Nym.surface)
        }
    }

    var socialsFootnote: some View {
        Text("signUp.socialsFootnote".localizedString)
            .nymTextStyle(.bodySmall)
            .foregroundStyle(Color.Nym.textSecondary)
            .multilineTextAlignment(.center)
    }
}
