import UIKit

public extension UIScreen {
    static var current: UIScreen? {
        UIWindow.current?.screen
    }
}
