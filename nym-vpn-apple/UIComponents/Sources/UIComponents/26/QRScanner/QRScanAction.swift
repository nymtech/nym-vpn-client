import Foundation

public struct QRScanAction {
    public let label: String
    public let action: () -> Void

    public init(label: String, action: @escaping () -> Void) {
        self.label = label
        self.action = action
    }
}
