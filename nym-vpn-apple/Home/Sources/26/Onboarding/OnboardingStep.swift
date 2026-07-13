import SwiftUI
import Theme

enum OnboardingStep: Int, CaseIterable, Identifiable {
    case welcome
    case modes
    case censorship

    var id: Int { rawValue }

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
