import SwiftUI
import Theme

enum OnboardingStep: Int, CaseIterable, Identifiable {
    case welcome
    case dvpn
    case mixnet
    case censorship
    case plan

    var id: Int { rawValue }

    var imageName: String? {
        switch self {
        case .welcome, .plan:
            nil
        case .dvpn:
            "dvpnMode"
        case .mixnet:
            "mixnetMode"
        case .censorship:
            "censorshipResistance"
        }
    }

    var animationName: String? {
        switch self {
        case .welcome:
            "noise-line"
        case .dvpn, .mixnet, .censorship, .plan:
            nil
        }
    }

    /// Non-nil on the screens that present the mode segmented control.
    /// Those screens also place the title above the illustration.
    var speedMode: OneClickSpeedMode? {
        switch self {
        case .dvpn:
            .fast
        case .mixnet:
            .anonymous
        case .welcome, .censorship, .plan:
            nil
        }
    }

    var illustrationHeight: CGFloat {
        switch self {
        case .dvpn, .mixnet, .plan:
            Constants.modeIllustrationHeight
        case .welcome, .censorship:
            Constants.illustrationHeight
        }
    }

    var isTitleUppercased: Bool {
        switch self {
        case .dvpn, .mixnet, .censorship:
            true
        case .welcome, .plan:
            false
        }
    }

    var titleKey: String {
        switch self {
        case .welcome:
            "onboarding26.welcome.title"
        case .dvpn:
            "onboarding26.dvpn.title"
        case .mixnet:
            "onboarding26.mixnet.title"
        case .censorship:
            "onboarding26.censorship.title"
        case .plan:
            "onboarding26.plan.title"
        }
    }

    var taglineKey: String? {
        switch self {
        case .dvpn:
            "onboarding26.dvpn.tagline"
        case .mixnet:
            "onboarding26.mixnet.tagline"
        case .welcome, .censorship, .plan:
            nil
        }
    }

    func subtitle(pricing: OnboardingPlanPricing?) -> AttributedString {
        switch self {
        case .welcome:
            regular("onboarding26.welcome.subtitle") + bold("onboarding26.welcome.subtitleEmphasis")
        case .dvpn:
            regular("onboarding26.dvpn.privacy")
                + newLine
                + bold("onboarding26.dvpn.decentralizedBold")
                + regular("onboarding26.dvpn.decentralized")
                + newLine
                + bold("onboarding26.dvpn.amneziaBold")
                + regular("onboarding26.dvpn.amnezia")
        case .mixnet:
            regular("onboarding26.mixnet.anonymous")
                + bold("onboarding26.mixnet.anonymousBold")
                + regular("onboarding26.mixnet.anonymousSuffix")
                + newLine
                + regular("onboarding26.mixnet.hops")
                + bold("onboarding26.mixnet.mixnetBold")
                + regular("onboarding26.mixnet.noise")
                + bold("onboarding26.mixnet.noiseBold")
                + newLine
                + regular("onboarding26.mixnet.patterns")
        case .censorship:
            regular("onboarding26.censorship.disguise")
                + bold("onboarding26.censorship.amneziaBold")
                + newLine
                + regular("onboarding26.censorship.firewalls")
                + bold("onboarding26.censorship.quicBold")
                + newLine
                + regular("onboarding26.censorship.blocking")
                + bold("onboarding26.censorship.stealthBold")
                + newLine
                + bold("onboarding26.censorship.postQuantumBold")
                + regular("onboarding26.censorship.postQuantum")
        case .plan:
            planSubtitle(pricing: pricing)
        }
    }
}

private extension OnboardingStep {
    /// Prices come from StoreKit — the priced lines are dropped when the products are unavailable.
    func planSubtitle(pricing: OnboardingPlanPricing?) -> AttributedString {
        var subtitle = bold("onboarding26.plan.communityFavorite") + newLine + newLine

        if let pricing {
            if let savings = pricing.savings {
                subtitle += bold("onboarding26.plan.yearPlan", color: Color.Nym.textPrimary)
                    + bold("onboarding26.plan.save", argument: savings, color: Color.Nym.primary)
                    + newLine
            }
            if let freeTrialPeriod = pricing.freeTrialPeriod {
                subtitle += bold("onboarding26.plan.freeTrial", argument: freeTrialPeriod) + newLine
            }
            subtitle += bold("onboarding26.plan.startingAt", color: Color.Nym.textPrimary)
                + bold("onboarding26.plan.price", argument: pricing.monthlyPrice, color: Color.Nym.primary)
                + newLine
                + newLine
        }

        subtitle += regular("onboarding26.plan.try")
            + bold("onboarding26.plan.oneMonth", color: Color.Nym.textPrimary)
            + regular("onboarding26.plan.trySuffix")
        return subtitle
    }
}

private extension OnboardingStep {
    enum Constants {
        static let illustrationHeight: CGFloat = 300
        static let modeIllustrationHeight: CGFloat = 163
    }

    var newLine: AttributedString {
        AttributedString("\n")
    }

    func regular(_ key: String, color: Color? = nil) -> AttributedString {
        var string = AttributedString(key.localizedString)
        string.foregroundColor = color
        return string
    }

    func bold(_ key: String, color: Color? = nil) -> AttributedString {
        var string = AttributedString(key.localizedString)
        string.font = .Nym.bodyDefaultBold
        string.foregroundColor = color
        return string
    }

    func bold(_ key: String, argument: CVarArg, color: Color? = nil) -> AttributedString {
        var string = AttributedString(String(format: key.localizedString, argument))
        string.font = .Nym.bodyDefaultBold
        string.foregroundColor = color
        return string
    }
}
