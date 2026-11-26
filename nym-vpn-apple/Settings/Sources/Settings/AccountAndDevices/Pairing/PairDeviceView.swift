import SwiftUI
import CredentialsManager
import ImpactGenerator
import Theme
import UIComponents

/// The presenter starts the session and verifies device auth on iOS; dismissal cancels it.
struct PairDeviceView: View {
    @ObservedObject var pairingManager: PairingManager
    let onClose: () -> Void

    var body: some View {
        TimelineView(.periodic(from: .now, by: 1)) { _ in
            VStack(spacing: 0) {
                header
                ScrollView {
                    VStack(spacing: NymSpacing.section) {
                        content
                    }
                    .padding(.horizontal, NymSpacing.section)
                    .padding(.vertical, NymSpacing.section)
                }
                .scrollIndicators(.never)
                buttons
                    .padding(.horizontal, NymSpacing.section)
                    .padding(.bottom, NymSpacing.section)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        // An accidental swipe would end the session the other device is joining.
        .interactiveDismissDisabled(isSessionLive)
#if os(iOS)
        .hidesWhenCaptured()
#elseif os(macOS)
        // A fixed size, so the sheet doesn't jump between states.
        .frame(width: 420, height: 560)
#endif
    }
}

private extension PairDeviceView {
    var remaining: Duration? {
        guard case let .showingCode(_, _, expiresAt) = pairingManager.state else { return nil }
        return max(.zero, expiresAt - .now)
    }

    var isExpired: Bool {
        pairingManager.state == .expired || remaining == .zero
    }

    var isSessionLive: Bool {
        switch pairingManager.state {
        case .showingCode:
            !isExpired
        case .confirmCode, .sharingAccount:
            true
        default:
            false
        }
    }

    var header: some View {
        VStack(spacing: NymSpacing.medium) {
            Image(systemName: "qrcode")
                .font(.system(size: 24))
                .foregroundStyle(Color.Nym.primary)
                .frame(width: 48, height: 48)
                .background(Color.Nym.primary.opacity(0.15), in: RoundedRectangle(cornerRadius: 12))
                .accessibilityHidden(true)
            Text("pairDevice.title".localizedString)
                .nymTextStyle(.titleScreen)
                .foregroundStyle(Color.Nym.textPrimary)
                .accessibilityAddTraits(.isHeader)
        }
        .padding(.top, NymSpacing.section)
    }

    @ViewBuilder
    var content: some View {
        switch pairingManager.state {
        case let .showingCode(url, _, _) where !isExpired:
            Text("pairDevice.instructions".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textSecondary)
                .multilineTextAlignment(.center)
            QRCodeImage(content: url, accessibilityLabel: "pairDevice.qrAccessibility".localizedString)
                .frame(maxWidth: Constants.qrMaxWidth)
            if let remaining {
                Text(String(format: "pairDevice.expiresIn".localizedString, remaining.minutesSeconds))
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .monospacedDigit()
            }
        case let .confirmCode(pin, _):
            PairingStatusView(
                title: "pairDevice.confirm.title".localizedString,
                message: "pairDevice.confirm.body".localizedString
            )
            PairingPinView(pin: pin)
        case .sharingAccount:
            PairingStatusView(title: "pairDevice.sharing".localizedString, isLoading: true)
        case .accountShared:
            PairingStatusView(
                title: "pairDevice.done.title".localizedString,
                message: "pairDevice.done.body".localizedString
            )
        case let .failed(error):
            PairingStatusView(title: error.localizedMessage)
        case .showingCode, .expired:
            PairingStatusView(title: "pairing.expired".localizedString)
        default:
            PairingStatusView(title: "pairDevice.starting".localizedString, isLoading: true)
        }
    }

    @ViewBuilder
    var buttons: some View {
        VStack(spacing: NymSpacing.medium) {
            switch pairingManager.state {
            case _ where isExpired:
                primaryButton("pairDevice.newCode".localizedString) { newCode() }
                secondaryButton("close".localizedString, action: close)
            case .failed:
                primaryButton("pairing.tryAgain".localizedString) { newCode() }
                secondaryButton("close".localizedString, action: close)
            case .confirmCode:
                primaryButton("pairDevice.codesMatch".localizedString) {
                    Task { await pairingManager.confirmCode() }
                }
                secondaryButton("pairDevice.codesDontMatch".localizedString, action: close)
            case .accountShared:
                primaryButton("done".localizedString, action: close)
            case .sharingAccount:
                EmptyView()
            default:
                secondaryButton("cancel".localizedString, action: close)
            }
        }
    }

    func primaryButton(_ title: String, action: @escaping () -> Void) -> some View {
        NymButton(title, style: .primary) {
            ImpactGenerator.shared.softImpact()
            action()
        }
    }

    func secondaryButton(_ title: String, action: @escaping () -> Void) -> some View {
        NymButton(title, style: .secondary) {
            ImpactGenerator.shared.softImpact()
            action()
        }
    }

    func newCode() {
        Task { await pairingManager.startShowingCode() }
    }

    func close() {
        Task { await pairingManager.cancel() }
        onClose()
    }

    enum Constants {
        static let qrMaxWidth: CGFloat = 240
    }
}

private extension Duration {
    var minutesSeconds: String {
        let seconds = max(0, Int(components.seconds))
        return String(format: "%d:%02d", seconds / 60, seconds % 60)
    }
}
