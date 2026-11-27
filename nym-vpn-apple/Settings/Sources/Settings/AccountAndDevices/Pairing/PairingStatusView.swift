import SwiftUI
import Theme
import UIComponents

struct PairingStatusView: View {
    let title: String
    var message: String?
    var isLoading = false

    var body: some View {
        VStack(spacing: NymSpacing.large) {
            if isLoading {
                ProgressView()
                    .tint(Color.Nym.primary)
                    .controlSize(.large)
            }
            Text(title)
                .nymTextStyle(.titleScreen)
                .foregroundStyle(Color.Nym.textPrimary)
                .multilineTextAlignment(.center)
                .accessibilityAddTraits(.isHeader)
            if let message {
                Text(message)
                    .nymTextStyle(.bodyDefault)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .multilineTextAlignment(.center)
            }
        }
        .frame(maxWidth: .infinity)
        .onAppear {
            AccessibilityNotification.Announcement(title).post()
        }
        .onChange(of: title) { _, newTitle in
            AccessibilityNotification.Announcement(newTitle).post()
        }
    }
}
