import Foundation

public enum FreepassResult: Equatable, Sendable {
    case redeemed
    case failed(FreepassError)
}
