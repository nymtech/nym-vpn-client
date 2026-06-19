import SwiftUI
import AccountPrefetchGates
import CredentialsManager
import Theme
import UIComponents

struct AuthFlowView: View {
    enum Step: Equatable {
        case welcome
        case signUp
        case signIn
    }

    let credentialsManager: CredentialsManager
    let sessionCoordinator: AppSessionCoordinating

    @State private var step: Step = .welcome
    @State private var cardHeight: CGFloat?
    @State private var welcomeRootHeight: CGFloat = 0
    @State private var signUpRootHeight: CGFloat = 0
    @State private var signInRootHeight: CGFloat = 0
    @State private var passphraseHeight: CGFloat = 0
    @State private var generateCarouselHeight: CGFloat = 0
    @State private var measurementPassphraseViewModel: PassphraseSignInViewModel

    init(
        credentialsManager: CredentialsManager,
        sessionCoordinator: AppSessionCoordinating
    ) {
        self.credentialsManager = credentialsManager
        self.sessionCoordinator = sessionCoordinator
        _measurementPassphraseViewModel = State(
            wrappedValue: PassphraseSignInViewModel(credentialsManager: credentialsManager)
        )
    }

    private var sharedRootHeight: CGFloat {
        AuthFlowHeightPolicy.sharedRootHeight(
            welcome: welcomeRootHeight,
            signUp: signUpRootHeight,
            signIn: signInRootHeight,
            passphrase: passphraseHeight,
            generateCarousel: generateCarouselHeight
        )
    }

    private var signInSharedHeight: CGFloat {
        AuthFlowHeightPolicy.signInRootHeight(
            welcome: welcomeRootHeight,
            signIn: signInRootHeight,
            passphrase: passphraseHeight
        )
    }

    var body: some View {
        ZStack(alignment: .top) {
            measurementLayer
            content
        }
        .frame(maxWidth: .infinity)
        .frame(height: cardHeight)
        .animation(.easeInOut, value: step)
        .clipped()
    }
}

private extension AuthFlowView {
    var measurementLayer: some View {
        ZStack {
            WelcomeView(onSignInTapped: {}, onSignUpTapped: {})
                .trackHeight { welcomeRootHeight = $0 }
            SignUpRootContent(
                privyLoadingTarget: nil,
                onBackTapped: {},
                onAnonymousTapped: {},
                onSocialsTapped: {}
            )
            .trackHeight { signUpRootHeight = $0 }
            SignInRootContent(
                isPrivyLoading: false,
                onBackTapped: {},
                onPassphraseTapped: {},
                onSocialsTapped: {}
            )
            .trackHeight { signInRootHeight = $0 }
            PassphraseSignInView(
                viewModel: measurementPassphraseViewModel,
                onBackTapped: {}
            )
            .trackHeight { passphraseHeight = $0 }
            GeneratePassphraseCarouselMeasurement()
                .trackHeight { generateCarouselHeight = $0 }
        }
        .fixedSize(horizontal: false, vertical: true)
        .hidden()
        .accessibilityHidden(true)
        .allowsHitTesting(false)
    }

    @ViewBuilder
    var content: some View {
        switch step {
        case .welcome:
            WelcomeView(
                minHeight: sharedRootHeight,
                onSignInTapped: { step = .signIn },
                onSignUpTapped: { step = .signUp }
            )
            .fixedSize(horizontal: false, vertical: true)
            .trackHeight { cardHeight = $0 }
            .transition(.slideFade(from: .leading))
        case .signUp:
            SignUpView(
                credentialsManager: credentialsManager,
                sessionCoordinator: sessionCoordinator,
                rootMinHeight: sharedRootHeight,
                onBackTapped: { step = .welcome }
            )
            .fixedSize(horizontal: false, vertical: true)
            .trackHeight { cardHeight = $0 }
            .transition(.slideFade(from: .trailing))
        case .signIn:
            SignInView(
                credentialsManager: credentialsManager,
                sessionCoordinator: sessionCoordinator,
                rootMinHeight: signInSharedHeight,
                onBackTapped: { step = .welcome }
            )
            .fixedSize(horizontal: false, vertical: true)
            .trackHeight { cardHeight = $0 }
            .transition(.slideFade(from: .trailing))
        }
    }
}

private struct GeneratePassphraseCarouselMeasurement: View {
    var body: some View {
        VStack(spacing: AuthLayout.stackSpacing) {
            Spacer().frame(height: 27)
            StepView(stepCount: 4, currentStep: .constant(1))
            WaveDotsView()
            Spacer().frame(height: NymSpacing.large)
            VStack(spacing: 16) {
                ForEach(1...3, id: \.self) { index in
                    VStack(alignment: .center, spacing: 16) {
                        Text("generatePassphrase.title\(index)".localizedString)
                            .textStyle(.Headline.Medium.regular)
                            .multilineTextAlignment(.center)
                        Text("generatePassphrase.subtitle\(index)".localizedString)
                            .textStyle(.Body.Medium.regular)
                            .multilineTextAlignment(.center)
                    }
                }
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, NymSpacing.component)
        .padding(.vertical, AuthLayout.verticalPadding)
    }
}
