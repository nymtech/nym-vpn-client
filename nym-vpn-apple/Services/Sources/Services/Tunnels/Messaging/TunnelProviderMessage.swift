import Foundation

public enum TunnelProviderMessage: Codable {
    case status

    public init(messageData: Data) throws {
        self = try JSONDecoder().decode(Self.self, from: messageData)
    }

    public func encode() throws -> Data {
        try JSONEncoder().encode(self)
    }
}
