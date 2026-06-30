import Foundation

public enum AccountSummaryRefreshPolicy {
    public static func shouldForceNetworkRefresh(force: Bool, isAccountActive: Bool) -> Bool {
        force || !isAccountActive
    }

    public static func pollDelays(untilActive: Bool) -> [Duration] {
        _ = untilActive
        return [
            .zero,
            .seconds(1),
            .seconds(2),
            .seconds(3),
            .seconds(4),
            .seconds(6),
            .seconds(10)
        ]
    }
}
