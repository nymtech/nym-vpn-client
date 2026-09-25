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
    let older = local.compare(floor, options: .numeric) == .orderedAscending
    guard older else {
        return AppUpdatePrompt(show: false, blockConnect: false)
    }
    return AppUpdatePrompt(show: true, blockConnect: policy == "required")
}
