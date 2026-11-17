#if os(iOS)
import SwiftUI
import Theme

public extension View {
    /// iOS can't block screenshots, so the content is hidden while recorded or mirrored.
    func hidesWhenCaptured() -> some View {
        modifier(CaptureProtectionModifier())
    }
}

private struct CaptureProtectionModifier: ViewModifier {
    @State private var isCaptured = Self.isScreenCaptured

    func body(content: Content) -> some View {
        content
            .accessibilityHidden(isCaptured)
            .overlay {
                if isCaptured {
                    ZStack {
                        Color.Nym.background
                        Text("captureProtection.hidden".localizedString)
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.Nym.textSecondary)
                            .multilineTextAlignment(.center)
                            .padding(NymSpacing.section)
                    }
                    .ignoresSafeArea()
                }
            }
            .onReceive(NotificationCenter.default.publisher(for: UIScreen.capturedDidChangeNotification)) { _ in
                isCaptured = Self.isScreenCaptured
            }
    }

    private static var isScreenCaptured: Bool {
        UIApplication.shared.connectedScenes
            .compactMap { $0 as? UIWindowScene }
            .contains { $0.traitCollection.sceneCaptureState == .active }
    }
}
#endif
