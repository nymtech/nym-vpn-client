import Foundation

public enum QRScanValidation: Equatable, Sendable {
    case valid
    /// Without `retryAfter`, the same code is rejected for as long as it stays in view.
    case invalid(message: String? = nil, retryAfter: Duration? = nil)
}
