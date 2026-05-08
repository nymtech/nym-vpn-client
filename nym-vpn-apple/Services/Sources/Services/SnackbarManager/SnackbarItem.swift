import Foundation

public struct SnackbarItem: Identifiable {
    public enum Style: Equatable, Sendable {
        case critical
        case confirmation
        case neutral
        case negative
        case warning
    }

    public let id: UUID
    public var style: Style
    public var title: String
    public var message: String?
    public var actionTitle: String?
    public var onAction: (@MainActor () -> Void)?
    public var duration: TimeInterval?

    public init(
        id: UUID = UUID(),
        style: Style,
        title: String,
        message: String? = nil,
        actionTitle: String? = nil,
        onAction: (@MainActor () -> Void)? = nil,
        duration: TimeInterval? = 4
    ) {
        self.id = id
        self.style = style
        self.title = title
        self.message = message
        self.actionTitle = actionTitle
        self.onAction = onAction
        self.duration = duration
    }
}
