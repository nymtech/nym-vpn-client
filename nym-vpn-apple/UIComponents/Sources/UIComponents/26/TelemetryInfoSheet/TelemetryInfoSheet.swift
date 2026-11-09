import SwiftUI
import Theme

/// Content for the "What's shared" telemetry info sheet. Built by the caller
/// (which resolves the localized strings) so this component stays string-agnostic,
/// matching the rest of UIComponents.
public struct TelemetryInfoContent: Identifiable {
    public let id: String
    public let title: String
    public let bullets: [String]
    public let neverLabel: String
    public let never: String
    public let handling: String
    public let gotItTitle: String
    public let privacyLinkTitle: String
    public let privacyURL: URL?

    public init(
        id: String,
        title: String,
        bullets: [String],
        neverLabel: String,
        never: String,
        handling: String,
        gotItTitle: String,
        privacyLinkTitle: String,
        privacyURL: URL?
    ) {
        self.id = id
        self.title = title
        self.bullets = bullets
        self.neverLabel = neverLabel
        self.never = never
        self.handling = handling
        self.gotItTitle = gotItTitle
        self.privacyLinkTitle = privacyLinkTitle
        self.privacyURL = privacyURL
    }
}

/// Shared "What's shared" sheet for the telemetry opt-in toggles, used by the
/// onboarding opt-in screen and the Data & privacy settings screen.
public struct TelemetryInfoSheet: View {
    private let content: TelemetryInfoContent
    private let onDismiss: () -> Void

    public init(content: TelemetryInfoContent, onDismiss: @escaping () -> Void) {
        self.content = content
        self.onDismiss = onDismiss
    }

    public var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Image(systemName: "info.circle")
                .foregroundStyle(Color.Nym.textPrimary)
                .frame(maxWidth: .infinity, alignment: .center)

            Text(content.title)
                .nymTextStyle(.bodyDefaultBold)
                .foregroundStyle(Color.Nym.textPrimary)
                .frame(maxWidth: .infinity, alignment: .center)

            VStack(alignment: .leading, spacing: 8) {
                ForEach(content.bullets, id: \.self) { bullet in
                    HStack(alignment: .top, spacing: 8) {
                        Text("•")
                            .nymTextStyle(.bodySmall)
                            .foregroundStyle(Color.Nym.textSecondary)
                        Text(bullet)
                            .nymTextStyle(.bodySmall)
                            .foregroundStyle(Color.Nym.textSecondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                    }
                }
            }

            Divider()

            VStack(alignment: .leading, spacing: 2) {
                Text(content.neverLabel)
                    .nymTextStyle(.bodySmall)
                    .fontWeight(.semibold)
                    .foregroundStyle(Color.Nym.textPrimary)
                Text(content.never)
                    .nymTextStyle(.bodySmall)
                    .foregroundStyle(Color.Nym.textSecondary)
            }

            Text(content.handling)
                .nymTextStyle(.bodySmall)
                .foregroundStyle(Color.Nym.textSecondary)

            Spacer()

            VStack(spacing: 12) {
                NymButton(content.gotItTitle, style: .primary) {
                    onDismiss()
                }
                if let url = content.privacyURL {
                    Link(destination: url) {
                        Text(content.privacyLinkTitle)
                            .nymTextStyle(.bodySmall)
                            .foregroundStyle(Color.Nym.info)
                            .underline()
                    }
                }
            }
        }
        .padding(24)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
        .background(Color.Nym.background.ignoresSafeArea())
        #if os(macOS)
        .frame(minWidth: 380, idealWidth: 420, minHeight: 440, idealHeight: 540)
        #endif
    }
}
