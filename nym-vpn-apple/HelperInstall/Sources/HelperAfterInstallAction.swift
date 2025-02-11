import Foundation

public final class HelperAfterInstallAction: Hashable, Equatable, Identifiable {
    public var id: String = UUID().uuidString
    public var completion: (@Sendable @MainActor () -> Void)?

    public init(completion: (@Sendable @MainActor () -> Void)?) {
        self.completion = completion
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    public static func == (lhs: HelperAfterInstallAction, rhs: HelperAfterInstallAction) -> Bool {
        lhs.id == rhs.id
    }
}
