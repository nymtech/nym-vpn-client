import Foundation

@objc public protocol UpdaterProtocol {
    func killHelper(completion: @escaping (Bool, String?) -> Void)
}
