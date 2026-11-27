#if os(iOS)
import SwiftUI
import CredentialsManager
import ImpactGenerator
import Theme
import UIComponents

struct PairingSignInView: View {
    @ObservedObject var pairingManager: PairingManager
    let onSignedIn: () -> Void
    let onBack: () -> Void

    var body: some View {
        ScrollView {
            VStack(spacing: NymSpacing.section) {
                content
            }
            .padding(.horizontal, NymSpacing.large)
            .padding(.vertical, NymSpacing.section)
            .padding(.top, Constants.topInset)
        }
        .scrollIndicators(.never)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .task(id: pairingManager.state) {
            if pairingManager.state == .signedIn {
                onSignedIn()
            }
        }
    }
}

private extension PairingSignInView {
    @ViewBuilder
    var content: some View {
        switch pairingManager.state {
        case .waitingForCode:
            PairingStatusView(title: "pairing.waitingForCode".localizedString, isLoading: true)
            cancelButton
        case let .waitingForApproval(pin, _):
            PairingStatusView(
                title: "pairing.signIn.title".localizedString,
                message: "pairing.signIn.body".localizedString
            )
            PairingPinView(pin: pin)
            ProgressView()
                .tint(Color.Nym.primary)
            cancelButton
        case .signedIn:
            PairingStatusView(title: "pairing.signingIn".localizedString, isLoading: true)
        case .expired:
            ended(title: "pairing.expired".localizedString)
        case let .failed(error):
            ended(title: error.localizedMessage)
        default:
            ended(title: "pairing.ended".localizedString)
        }
    }

    var cancelButton: some View {
        NymButton("cancel".localizedString, style: .textOnly) {
            ImpactGenerator.shared.softImpact()
            onBack()
        }
    }

    @ViewBuilder
    func ended(title: String) -> some View {
        PairingStatusView(title: title)
        NymButton("back".localizedString, style: .primary) {
            ImpactGenerator.shared.softImpact()
            onBack()
        }
    }

    enum Constants {
        static let topInset: CGFloat = 48
    }
}
#endif
