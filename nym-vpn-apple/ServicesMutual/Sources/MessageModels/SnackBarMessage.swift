public struct SnackBarMessage {
    public let text: String
    public let subtitle: String?
    public let ctaText: String?
    public let ctaAction: (() -> Void)?
    public let closeAction: (() -> Void)?
    public let style: SnackbarStyle
    public let priority: BannerPriority

    public init(
        text: String,
        style: SnackbarStyle,
        subtitle: String? = nil,
        ctaText: String? = nil,
        ctaAction: (() -> Void)? = nil,
        closeAction: (() -> Void)? = nil,
        priority: BannerPriority = .normal
    ) {
        self.text = text
        self.style = style
        self.subtitle = subtitle
        self.ctaText = ctaText
        self.ctaAction = ctaAction
        self.closeAction = closeAction
        self.priority = priority
    }
}
