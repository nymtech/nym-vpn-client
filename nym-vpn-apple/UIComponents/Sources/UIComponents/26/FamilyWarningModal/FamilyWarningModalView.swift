import SwiftUI
import Theme

public struct FamilyWarningModalView: View {
    private let title: String
    private let reminderText: String
    private let reminderLinkText: String
    private let connectAnywayTitle: String
    private let cancelTitle: String
    private let onConnectAnyway: () -> Void
    private let onCancel: () -> Void
    private let onOpenNotificationSettings: () -> Void

    private let notificationsLinkURL = URL(string: "app://open-notifications")

    public init(
        title: String,
        reminderText: String,
        reminderLinkText: String,
        connectAnywayTitle: String,
        cancelTitle: String,
        onConnectAnyway: @escaping () -> Void,
        onCancel: @escaping () -> Void,
        onOpenNotificationSettings: @escaping () -> Void
    ) {
        self.title = title
        self.reminderText = reminderText
        self.reminderLinkText = reminderLinkText
        self.connectAnywayTitle = connectAnywayTitle
        self.cancelTitle = cancelTitle
        self.onConnectAnyway = onConnectAnyway
        self.onCancel = onCancel
        self.onOpenNotificationSettings = onOpenNotificationSettings
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(alignment: .top, spacing: 12) {
                Image(systemName: "exclamationmark.triangle.fill")
                    .font(.system(size: 20))
                    .foregroundStyle(Color.Nym.primary)
                Text(title)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyDefaultBold)
                    .fixedSize(horizontal: false, vertical: true)
            }

            Text(reminderAttributedText())
                .italic()
                .nymTextStyle(.bodySmall)
                .lineLimit(1)
                .minimumScaleFactor(0.6)
                .frame(maxWidth: .infinity, alignment: .leading)
                .tint(Color.Nym.primary)
                .environment(\.openURL, OpenURLAction { url in
                    guard url == notificationsLinkURL else { return .systemAction }
                    onOpenNotificationSettings()
                    return .handled
                })
                .padding(.top, 12)

            HStack(spacing: 12) {
                NymButton(
                    cancelTitle,
                    style: .secondary,
                    cornerRadius: 28,
                    foregroundColor: .Nym.textSecondary,
                    borderColor: .Nym.textSecondary,
                    action: onCancel
                )
                NymButton(
                    connectAnywayTitle,
                    style: .primary,
                    cornerRadius: 28,
                    action: onConnectAnyway
                )
            }
            .padding(.top, 24)
        }
        .padding(20)
    }

    private func reminderAttributedText() -> AttributedString {
        var attributed = AttributedString(reminderText)
        attributed.foregroundColor = Color.Nym.textSecondary
        if let range = attributed.range(of: reminderLinkText) {
            attributed[range].foregroundColor = Color.Nym.primary
            attributed[range].underlineStyle = .single
            attributed[range].link = notificationsLinkURL
        }
        return attributed
    }
}
