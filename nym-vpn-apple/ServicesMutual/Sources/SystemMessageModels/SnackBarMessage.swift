public struct SnackBarMessage {
    public let text: String
    public let style: SnackbarStyle

    public init(text: String, style: SnackbarStyle) {
        self.text = text
        self.style = style
    }
}
