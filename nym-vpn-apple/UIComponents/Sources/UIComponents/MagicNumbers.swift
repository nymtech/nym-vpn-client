import Foundation
import Device

public enum MagicNumbers: CGFloat {
    case macMinWidth = 390
    case macMinHeight = 675
    case ipadMaxWidth = 358

    public static var maxWidth: CGFloat {
        switch Device.type {
        case .ipad:
            358
        case .iphone:
            .infinity
        case .mac:
            390
        }
    }
}
