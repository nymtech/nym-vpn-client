#if os(macOS)
import NymVPNRpc
#endif

public struct SplitTunnelConfig: Codable, Equatable {
    public var isEnabled: Bool
    public var appPaths: [String]

    public init(
        isEnabled: Bool = false,
        appPaths: [String] = []
    ) {
        self.isEnabled = isEnabled
        self.appPaths = appPaths
    }

#if os(macOS)
    public init(from settings: SplitTunnelSettings) {
        self.isEnabled = settings.enabled
        self.appPaths = settings.apps.map { $0.path }
    }
#endif

    public func diff(comparedTo newConfig: SplitTunnelConfig) -> (added: [String], removed: [String]) {
        let currentPaths = Set(appPaths)
        let newPaths = Set(newConfig.appPaths)

        let added = Array(newPaths.subtracting(currentPaths)).sorted()
        let removed = Array(currentPaths.subtracting(newPaths)).sorted()

        return (added: added, removed: removed)
    }
}
