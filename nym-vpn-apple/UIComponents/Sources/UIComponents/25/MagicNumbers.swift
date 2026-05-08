import Foundation
import Device

public enum MagicNumbers: CGFloat {
    case macMinWidth = 390
    case macMinHeight = 750
    case ipadMaxWidth = 358
    case ipadExtraWidth = 450

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

    public static var moreMaxWidth: CGFloat {
        switch Device.type {
        case .ipad:
            450
        case .iphone:
                .infinity
        case .mac:
            450
        }
    }
}
