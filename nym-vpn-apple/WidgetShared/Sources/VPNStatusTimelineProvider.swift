import WidgetKit
import NetworkExtension

public struct VPNStatusTimelineProvider: TimelineProvider {
    public typealias Entry = VPNStatusTimelineEntry

    private static let groupDefaults = UserDefaults(suiteName: "group.net.nymtech.vpn")
    #if os(iOS)
    private static let entryKey = "ios_widgetEntryLocation"
    private static let exitKey = "ios_widgetExitLocation"
    #elseif os(macOS)
    private static let entryKey = "macos_widgetEntryLocation"
    private static let exitKey = "macos_widgetExitLocation"
    private static let statusKey = "macos_widgetTunnelStatus"
    #endif

    public init() {}

    public func placeholder(in context: Context) -> VPNStatusTimelineEntry {
        VPNStatusTimelineEntry(
            date: Date(),
            status: .status(.connected),
            entryLocation: "Switzerland",
            exitLocation: "France"
        )
    }

    public func getSnapshot(in context: Context, completion: @escaping (Entry) -> Void) {
        completion(placeholder(in: context))
    }

    public func getTimeline(in context: Context, completion: @escaping (Timeline<Entry>) -> Void) {
        Task {
            let entry = await buildEntry()
            completion(Timeline(entries: [entry], policy: .atEnd))
        }
    }
}

private extension VPNStatusTimelineProvider {
    func buildEntry() async -> VPNStatusTimelineEntry {
        let defaults = Self.groupDefaults
        defaults?.synchronize()
        let entryLoc = defaults?.string(forKey: Self.entryKey) ?? ""
        let exitLoc = defaults?.string(forKey: Self.exitKey) ?? ""

        let status: VPNStatus
#if os(iOS)
        do {
            if let manager = try await NymTunnelManager.loadManager() {
                status = .status(manager.connection.status)
            } else {
                status = .notConfigured
            }
        } catch {
            status = .error
        }
#elseif os(macOS)
        if let rawValue = defaults?.object(forKey: Self.statusKey) as? Int {
            status = VPNStatus(tunnelStatusRawValue: rawValue)
        } else {
            status = .notConfigured
        }
#endif

        return VPNStatusTimelineEntry(
            date: Date(),
            status: status,
            entryLocation: entryLoc,
            exitLocation: exitLoc
        )
    }
}
