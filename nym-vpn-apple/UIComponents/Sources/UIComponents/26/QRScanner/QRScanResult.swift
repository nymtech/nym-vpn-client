import Foundation

public struct QRScanResult: Equatable, Sendable, CustomStringConvertible, CustomDebugStringConvertible {
    public let contents: String

    public init(contents: String) {
        self.contents = contents
    }

    // Keeps the scanned content out of logs.
    public var description: String {
        "QRScanResult(contents: <redacted>)"
    }

    public var debugDescription: String {
        description
    }
}
