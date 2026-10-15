import Foundation

struct AppUpdatePrompt: Equatable {
    var show: Bool
    var blockConnect: Bool
}

func appUpdatePrompt(
    local: String?,
    floor: String?,
    policy: String?,
    policies: [String: String] = [:],
    platform: String = ""
) -> AppUpdatePrompt {
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
    // This platform's own policy wins over the default.
    return AppUpdatePrompt(show: true, blockConnect: (policies[platform] ?? policy) == "required")
}
