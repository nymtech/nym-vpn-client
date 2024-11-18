import Foundation

public struct NymNetworkMessage {
    public var name: String
    public var message: String
    public var properties: [String: String]

    public init(name: String, message: String, properties: [String: String]) {
        self.name = name
        self.message = message
        self.properties = properties
    }
}
