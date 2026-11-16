#if os(iOS)
import SwiftUI
import Theme

struct QRScannerMessageView: View {
    let systemImageName: String
    let title: String
    let message: String
    var primaryAction: QRScanAction?
    var secondaryAction: QRScanAction?

    var body: some View {
        VStack(spacing: NymSpacing.component) {
            ScrollView {
                VStack(spacing: NymSpacing.large) {
                    Image(systemName: systemImageName)
                        .font(.system(size: 40))
                        .foregroundStyle(Color.white)
                        .accessibilityHidden(true)
                    Text(title)
                        .nymTextStyle(.titleScreen)
                        .foregroundStyle(Color.white)
                        .multilineTextAlignment(.center)
                        .accessibilityAddTraits(.isHeader)
                    Text(message)
                        .nymTextStyle(.bodyDefault)
                        .foregroundStyle(Color.white.opacity(0.8))
                        .multilineTextAlignment(.center)
                }
                .frame(maxWidth: .infinity)
                .padding(.top, Constants.topInset)
            }
            .scrollIndicators(.never)
            if let primaryAction {
                NymButton(primaryAction.label, style: .primary, action: primaryAction.action)
            }
            if let secondaryAction {
                NymButton(
                    secondaryAction.label,
                    style: .textOnly,
                    foregroundColor: .white,
                    action: secondaryAction.action
                )
            }
        }
        .padding(.horizontal, NymSpacing.section)
        .padding(.bottom, NymSpacing.section)
    }

    private enum Constants {
        static let topInset: CGFloat = 96
    }
}
#endif
