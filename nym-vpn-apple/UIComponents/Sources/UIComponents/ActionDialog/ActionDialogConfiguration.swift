import Theme

public struct ActionDialogConfiguration {
    let iconImageName: String?
    let titleLocalizedString: String?
    let subtitleLocalizedString: String?
    let yesLocalizedString: String?
    let noLocalizedString: String?
    let yesAction: (() -> Void)?
    let noAction: (() -> Void)?

    public init (
        iconImageName: String? = nil,
        titleLocalizedString: String? = nil,
        subtitleLocalizedString: String? = nil,
        yesLocalizedString: String? = nil,
        noLocalizedString: String? = nil,
        yesAction: (() -> Void)? = nil,
        noAction: (() -> Void)? = nil
    ) {
        self.iconImageName = iconImageName
        self.titleLocalizedString = titleLocalizedString
        self.subtitleLocalizedString = subtitleLocalizedString
        self.yesLocalizedString = yesLocalizedString
        self.noLocalizedString = noLocalizedString
        self.yesAction = yesAction
        self.noAction = noAction
    }
}
