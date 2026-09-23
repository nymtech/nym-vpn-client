import Foundation
import NymVPNLib

/// Built-in connection presets, mirroring the core's `Profile` type. Selecting one
/// reconfigures entry/exit/hop-count/stealth mode on the core; it doesn't connect by itself.
public enum ConnectionProfile: String, Codable, CaseIterable, Equatable, Sendable {
    case safest
    case mostPrivate
    case fastest
    case random
}

extension ConnectionProfile {
    public var imageName: String {
        switch self {
        case .safest:
            "safest"
        case .mostPrivate:
            "private"
        case .fastest:
            "fastest"
        case .random:
            "random"
        }
    }

    /// Resolve with `.localizedString`.
    public var titleKey: String {
        switch self {
        case .safest:
            "profiles.safest.title"
        case .mostPrivate:
            "profiles.mostPrivate.title"
        case .fastest:
            "profiles.fastest.title"
        case .random:
            "profiles.random.title"
        }
    }

    /// Resolve with `.localizedString`.
    public var descriptionKey: String {
        switch self {
        case .safest:
            "profiles.safest.description"
        case .mostPrivate:
            "profiles.mostPrivate.description"
        case .fastest:
            "profiles.fastest.description"
        case .random:
            "profiles.random.description"
        }
    }
}

extension ConnectionProfile {
    // Mirrors the core's ProfileSpecifics::from(profile) (nym-vpn-lib/src/service/config/profile.rs).
    public var entry: EntryGateway {
        switch self {
        case .safest, .mostPrivate:
            .auto(excludeUserCountry: true)
        case .fastest:
            .auto(excludeUserCountry: false)
        case .random:
            .random
        }
    }

    public var exit: ExitRouter {
        self == .random ? .random : .auto
    }

    public var enableTwoHop: Bool {
        self != .mostPrivate
    }

    public var stealthMode: Bool {
        self == .safest
    }
}

extension ConnectionProfile {
    public func toCoreProfile() -> NymVPNLib.Profile {
        switch self {
        case .safest:
            .safest
        case .mostPrivate:
            .mostPrivate
        case .fastest:
            .fastest
        case .random:
            .random
        }
    }
}
