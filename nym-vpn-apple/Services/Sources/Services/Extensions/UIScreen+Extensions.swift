import UIKit

public extension UIScreen {
    public static var current: UIScreen? {
        UIWindow.current?.screen
    }
}
