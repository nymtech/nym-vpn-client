#if os(iOS)
import UIKit
#endif

public enum Device {
    public enum DeviceType {
        case iphone
        case ipad
        case mac
    }

    public static var type: DeviceType {
#if os(macOS)
        return .mac
#else
        return UIDevice.current.userInterfaceIdiom == .pad ? .ipad : .iphone
#endif
    }

    public static var isMacOS: Bool {
#if os(macOS)
        return true
#else
        return false
#endif
    }
}
