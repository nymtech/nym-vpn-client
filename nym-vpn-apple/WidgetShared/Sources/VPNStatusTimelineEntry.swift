import WidgetKit

public struct VPNStatusTimelineEntry: TimelineEntry {
    public let date: Date
    public let status: VPNStatus
    public let entryLocation: String
    public let exitLocation: String

    public init(date: Date, status: VPNStatus = .notConfigured, entryLocation: String = "", exitLocation: String = "") {
        self.date = date
        self.status = status
        self.entryLocation = entryLocation
        self.exitLocation = exitLocation
    }
}
