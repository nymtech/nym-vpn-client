public struct GenericBannerViewConfig {
    public let title: String
    public let subtitle: String
    public let actionTitle: String
    public let action: () -> Void

    public init(title: String, subtitle: String, actionTitle: String, action: @escaping () -> Void) {
        self.title = title
        self.subtitle = subtitle
        self.actionTitle = actionTitle
        self.action = action
    }
}
