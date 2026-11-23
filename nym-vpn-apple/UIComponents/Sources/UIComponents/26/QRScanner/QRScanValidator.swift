import Foundation

@MainActor
public protocol QRScanValidator: AnyObject {
    func validate(_ contents: String) async throws -> QRScanValidation
}
