import Foundation

public struct SnackbarItem: Identifiable {
    public enum Style: Equatable, Sendable {
        case critical
        case confirmation
        case neutral
        case negative
        case warning

        /// Close control is always available. Critical used to hide it, which left
        /// macOS users stuck on login/processing errors with no dismiss control.
        public var showsCloseButton: Bool { true }
    }

    public let id: UUID
    public var style: Style
    public var title: String
    public var message: String?
    public var actionTitle: String?
    public var onAction: (@MainActor () -> Void)?
    public var secondaryActionTitle: String?
    public var onSecondaryAction: (@MainActor () -> Void)?
    /// Invoked when the user explicitly dismisses the snackbar via its close
    /// control. Not called when the snackbar is cleared programmatically (e.g. a
    /// connection flow calling `clear()`), so callers can distinguish an explicit
    /// dismissal from an incidental teardown.
    public var onDismiss: (@MainActor () -> Void)?
    public var duration: TimeInterval?

    public init(
        id: UUID = UUID(),
        style: Style = .neutral,
        title: String = "",
        message: String? = nil,
        actionTitle: String? = nil,
        onAction: (@MainActor () -> Void)? = nil,
        secondaryActionTitle: String? = nil,
        onSecondaryAction: (@MainActor () -> Void)? = nil,
        onDismiss: (@MainActor () -> Void)? = nil,
        duration: TimeInterval? = 4
    ) {
        self.id = id
        self.style = style
        self.title = title
        self.message = message
        self.actionTitle = actionTitle
        self.onAction = onAction
        self.secondaryActionTitle = secondaryActionTitle
        self.onSecondaryAction = onSecondaryAction
        self.onDismiss = onDismiss
        self.duration = duration
    }
}
