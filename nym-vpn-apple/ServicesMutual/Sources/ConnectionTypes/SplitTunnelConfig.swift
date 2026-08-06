#if os(macOS)
import NymVPNLib
#endif

public struct SplitTunnelConfig: Codable, Equatable {
    public var isEnabled: Bool
    public var appPaths: Set<String>
    /// Executable paths of apps the user added manually via the file picker.
    /// These are not produced by `AppDiscoveryService.enumerateApps()`, so they
    /// are persisted here to keep them visible in the list. UI-only — not sent
    /// to the daemon (only `appPaths` is synced).
    public var customAppPaths: Set<String>

    public init(
        isEnabled: Bool = false,
        appPaths: Set<String> = [],
        customAppPaths: Set<String> = []
    ) {
        self.isEnabled = isEnabled
        self.appPaths = appPaths
        self.customAppPaths = customAppPaths
    }

#if os(macOS)
    public init(from settings: SplitTunnelSettings) {
        self.isEnabled = settings.enabled
        self.appPaths = Set(settings.apps.map { $0.path })
        self.customAppPaths = []
    }
#endif

    enum CodingKeys: String, CodingKey {
        case isEnabled
        case appPaths
        case customAppPaths
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        isEnabled = try container.decode(Bool.self, forKey: .isEnabled)
        appPaths = try container.decode(Set<String>.self, forKey: .appPaths)
        customAppPaths = try container.decodeIfPresent(Set<String>.self, forKey: .customAppPaths) ?? []
    }

    public func diff(comparedTo newConfig: SplitTunnelConfig) -> (added: [String], removed: [String]) {
        let added = newConfig.appPaths.subtracting(appPaths).sorted()
        let removed = appPaths.subtracting(newConfig.appPaths).sorted()

        return (added: added, removed: removed)
    }
}
