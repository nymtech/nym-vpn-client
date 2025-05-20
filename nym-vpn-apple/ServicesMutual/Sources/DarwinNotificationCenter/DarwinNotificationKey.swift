import Foundation
import Constants

public enum DarwinNotificationKey: String {
    case reconfigureLogs

    public var key: String {
        Constants.groupID.rawValue + self.rawValue
    }
}
