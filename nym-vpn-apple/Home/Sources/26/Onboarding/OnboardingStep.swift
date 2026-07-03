import SwiftUI
import Theme

/// The ordered feature-overview screens shown in the 2.0 onboarding carousel.
/// Mirrors the Figma "Onboarding screens" flow (node 4440-20189).
enum OnboardingStep: Int, CaseIterable, Identifiable {
    case welcome
    case modes
    case censorship

    var id: Int { rawValue }

    /// Illustration asset in `Assets.xcassets/2025/Onboarding`.
    /// `nil` for steps that render bespoke content (or no illustration).
    var imageName: String? {
        switch self {
        case .welcome:
            nil
        case .modes:
            nil
        case .censorship:
            "censorshipResistance"
        }
    }

    var titleKey: String {
        switch self {
        case .welcome:
            "onboarding26.welcome.title"
        case .modes:
            "onboarding26.modes.title"
        case .censorship:
            "onboarding26.censorship.title"
        }
    }

    /// Localized subtitle with per-run emphasis, following the Figma copy.
    /// Bold runs use `.Nym.bodyDefaultBold`; regular runs inherit `.bodyDefault`.
    var subtitle: AttributedString {
        switch self {
        case .welcome:
            regular("onboarding26.welcome.subtitle") + bold("onboarding26.welcome.subtitleEmphasis")
        case .modes:
            bold("onboarding26.modes.subtitleFastBold")
                + regular("onboarding26.modes.subtitleFast")
                + AttributedString("\n\n")
                + regular("onboarding26.modes.subtitleAnonymous")
        case .censorship:
            bold("onboarding26.censorship.amnesiaBold")
                + regular("onboarding26.censorship.amnesia")
                + AttributedString("\n")
                + bold("onboarding26.censorship.quicBold")
                + regular("onboarding26.censorship.quic")
                + AttributedString("\n")
                + bold("onboarding26.censorship.stealthBold")
                + regular("onboarding26.censorship.stealth")
        }
    }
}

private extension OnboardingStep {
    func regular(_ key: String) -> AttributedString {
        AttributedString(key.localizedString)
    }

    func bold(_ key: String) -> AttributedString {
        var string = AttributedString(key.localizedString)
        string.font = .Nym.bodyDefaultBold
        return string
    }
}
