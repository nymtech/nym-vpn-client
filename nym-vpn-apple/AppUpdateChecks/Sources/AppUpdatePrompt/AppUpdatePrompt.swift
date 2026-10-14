import Foundation

public struct AppUpdatePrompt: Equatable {
    public var show: Bool
    public var blockConnect: Bool

    public init(show: Bool, blockConnect: Bool) {
        self.show = show
        self.blockConnect = blockConnect
    }
}

public func appUpdatePrompt(local: String?, floor: String?, policy: String?) -> AppUpdatePrompt {
    guard let local, let floor else {
        return AppUpdatePrompt(show: false, blockConnect: false)
    }
    // "2026.13.0-beta.1" is older than "2026.13.0"; `.numeric` alone ranks it newer.
    let localParts = local.split(separator: "-", maxSplits: 1)
    let floorParts = floor.split(separator: "-", maxSplits: 1)
    guard let localBase = localParts.first, let floorBase = floorParts.first else {
        return AppUpdatePrompt(show: false, blockConnect: false)
    }
    let order = String(localBase).compare(String(floorBase), options: .numeric)
    let older = order == .orderedAscending || (order == .orderedSame && localParts.count > floorParts.count)
    guard older else {
        return AppUpdatePrompt(show: false, blockConnect: false)
    }
    return AppUpdatePrompt(show: true, blockConnect: policy == "required")
}
