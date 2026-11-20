#if os(iOS)
import SwiftUI
import CredentialsManager
import Theme
import UIComponents

public struct CodeEntryView: View {
    private enum Step {
        case scanner
        case typedEntry
        case pairing
    }

    private let onFinish: (CodeEntryOutcome) -> Void

    @ObservedObject private var pairingManager = PairingManager.shared
    @State private var step: Step
    @State private var isTypedEntryPresented = false
    @State private var validator: ScannedCodeValidator
    @State private var redeemViewModel: RedeemCodeViewModel

    public init(startsWithScanner: Bool, onFinish: @escaping (CodeEntryOutcome) -> Void) {
        self.onFinish = onFinish
        let validator = ScannedCodeValidator()
        _validator = State(wrappedValue: validator)
        _redeemViewModel = State(wrappedValue: RedeemCodeViewModel(validator: validator))
        _step = State(wrappedValue: startsWithScanner && QRScanner.isSupported ? .scanner : .typedEntry)
    }

    public var body: some View {
        content
            .hidesWhenCaptured()
            .onAppear {
                redeemViewModel.onAccepted = { onAccepted($0) }
            }
            .onDisappear {
                // Leaving ends a session that hasn't signed in.
                guard step == .pairing, pairingManager.state != .signedIn else { return }
                Task { await pairingManager.cancel() }
            }
    }
}

private extension CodeEntryView {
    @ViewBuilder
    var content: some View {
        switch step {
        case .scanner:
            QRScanner(
                validator: validator,
                secondaryAction: QRScanAction(label: "redeemCode.enterInstead".localizedString) {
                    isTypedEntryPresented = true
                },
                isPaused: isTypedEntryPresented,
                onResult: { result in
                    guard let code = ScannedQRCode.parse(result.contents) else { return }
                    onAccepted(code)
                },
                onClose: { onFinish(.cancelled) }
            )
            .sheet(
                isPresented: $isTypedEntryPresented,
                onDismiss: { redeemViewModel.reset() },
                content: {
                    RedeemCodeView(viewModel: redeemViewModel) {
                        isTypedEntryPresented = false
                    }
                    .presentationDetents([.medium, .large])
                    // A redeem in flight can't be cancelled, so the sheet stays open.
                    .interactiveDismissDisabled(redeemViewModel.isLoading)
                    .hidesWhenCaptured()
                }
            )
        case .typedEntry:
            RedeemCodeView(viewModel: redeemViewModel) {
                onFinish(.cancelled)
            }
        case .pairing:
            PairingSignInView(
                pairingManager: pairingManager,
                onSignedIn: { onFinish(.pairingSignedIn) },
                onBack: {
                    Task { await pairingManager.cancel() }
                    onFinish(.cancelled)
                }
            )
        }
    }

    func onAccepted(_ code: ScannedQRCode) {
        switch code {
        case .freepass:
            onFinish(.freepassRedeemed)
        case .pairing:
            isTypedEntryPresented = false
            step = .pairing
        }
    }
}
#endif
