public struct SnackBarMessage {
    public let text: String
    public let ctaText: String?
    public let ctaAction: (() -> Void)?
    public let style: SnackbarStyle

    public init(text: String, style: SnackbarStyle, ctaText: String? = nil, ctaAction: (() -> Void)? = nil) {
        self.text = text
        self.style = style
        self.ctaText = ctaText
        self.ctaAction = ctaAction
    }
}
