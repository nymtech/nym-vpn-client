import SwiftUI
import Theme

/// Shared opt-in toggle card for the telemetry screens (onboarding + Data &
/// privacy), so the two modules don't each duplicate the row layout. The info
/// button fires `onInfo`; the caller owns the sheet state it drives.
public struct TelemetryOptInCard: View {
    private let icon: String
    private let title: String
    private let subtitle: String
    @Binding private var isOn: Bool
    private let onInfo: () -> Void

    public init(
        icon: String,
        title: String,
        subtitle: String,
        isOn: Binding<Bool>,
        onInfo: @escaping () -> Void
    ) {
        self.icon = icon
        self.title = title
        self.subtitle = subtitle
        self._isOn = isOn
        self.onInfo = onInfo
    }

    public var body: some View {
        HStack(spacing: NymSpacing.small) {
            Image(systemName: icon)
                .foregroundStyle(Color.Nym.textSecondary)
                .frame(width: 24, height: 24)
                .accessibilityHidden(true)
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 6) {
                    Text(title)
                        .nymTextStyle(.bodyDefaultBold)
                        .foregroundStyle(Color.Nym.textPrimary)
                    Button {
                        onInfo()
                    } label: {
                        Image(systemName: "info.circle")
                            .foregroundStyle(Color.Nym.textSecondary)
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel(
                        Text(String(format: "welcomeOptIns.info.accessibility".localizedString, title))
                    )
                }
                Text(subtitle)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textSecondary)
            }
            .frame(maxWidth: .infinity, alignment: .leading)
            Toggle("", isOn: $isOn)
                .labelsHidden()
                .tint(Color.Nym.primary)
                .accessibilityLabel(Text(title))
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 16)
                .fill(Color.Nym.surfaceAlt)
        )
    }
}
