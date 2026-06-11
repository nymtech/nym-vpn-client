import SwiftUI
import AppSettings
import ConnectionManager
import CredentialsManager
import Routes
import Theme
import UIComponents

public struct ProcessingAccountView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var credentialsManager: CredentialsManager
    @Binding private var path: NavigationPath
    @State private var didFinishAnimatingText = false
    @State private var didSettleAccount = false
    @State private var errorMessage: String?
    @State private var currentStep = 1

    public var body: some View {
        VStack(alignment: .center, spacing: 0) {
            navbar
            Spacer()
                .frame(height: 24)

            StepView(stepCount: 4, currentStep: $currentStep)
            Spacer()
            dotsAnimationView
            Spacer()
                .frame(height: 16)
            statusTextView
            Spacer()
        }
        .frame(maxWidth: MagicNumbers.moreMaxWidth)
        .padding(16)
        .navigationBarBackButtonHidden(true)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .onChange(of: didFinishAnimatingText) { _, _ in
            advanceIfReady()
        }
        .onChange(of: didSettleAccount) { _, _ in
            advanceIfReady()
        }
        .task {
            await prepareAccount()
        }
    }

    public init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension ProcessingAccountView {
    var navbar: some View {
        CustomNavBar(useElevationBackground: true)
    }

    var dotsAnimationView: some View {
        WaveDotsView()
    }

    @ViewBuilder
    var statusTextView: some View {
        if let errorMessage {
            VStack(alignment: .center, spacing: 16) {
                Text(errorMessage)
                    .textStyle(.Body.Medium.regular)
                    .multilineTextAlignment(.center)
                NymButton("retry".localizedString, style: .primary) {
                    resetProcessingState()
                    Task { await prepareAccount() }
                }
            }
        } else {
            SwitchingTitlesView(
                pairs: [
                    ("processingAccount.title2".localizedString, "processingAccount.subtitle2".localizedString),
                    ("processingAccount.title3".localizedString, "processingAccount.subtitle3".localizedString),
                    ("processingAccount.title4".localizedString, "processingAccount.subtitle4".localizedString),
                    ("processingAccount.title5".localizedString, "processingAccount.subtitle5".localizedString)
                ],
                didFinishAnimating: $didFinishAnimatingText,
                timerDidTick: {
                    currentStep += 1
                }
            )
        }
    }
}

private extension ProcessingAccountView {
    func prepareAccount() async {
        didSettleAccount = false
        do {
            try await credentialsManager.prepareAccountForConnection(
                canPrefetchZkNyms: connectionManager.canPrefetchZkNymsFromApp
            )
            didSettleAccount = true
        } catch {
            errorMessage = "generalNymError.somethingWentWrong".localizedString
        }
    }

    func advanceIfReady() {
        guard didFinishAnimatingText, didSettleAccount else { return }
        if appSettings.welcomeScreenDidDisplay {
            path = .init()
        } else {
            path = .init([HomeLink.technicalOptIns])
        }
    }

    func resetProcessingState() {
        didFinishAnimatingText = false
        didSettleAccount = false
        errorMessage = nil
        currentStep = 1
    }

}
