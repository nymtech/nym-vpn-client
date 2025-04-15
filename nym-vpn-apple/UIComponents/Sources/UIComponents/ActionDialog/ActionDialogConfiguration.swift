import SwiftUI
import Theme

public struct ActionDialogConfiguration {
    let systemIconImageName: String?
    let systemIconImageColor: Color?
    let titleLocalizedString: String?
    let subtitleLocalizedString: String?
    let yesLocalizedString: String?
    let noLocalizedString: String?
    let yesAction: (() -> Void)?
    let noAction: (() -> Void)?
    let shouldCloseAfterYesAction: Bool

    public init (
        systemIconImageName: String? = nil,
        systemIconImageColor: Color? = nil,
        titleLocalizedString: String? = nil,
        subtitleLocalizedString: String? = nil,
        yesLocalizedString: String? = nil,
        noLocalizedString: String? = nil,
        yesAction: (() -> Void)? = nil,
        noAction: (() -> Void)? = nil,
        shouldCloseAfterYesAction: Bool = true
    ) {
        self.systemIconImageName = systemIconImageName
        self.systemIconImageColor = systemIconImageColor
        self.titleLocalizedString = titleLocalizedString
        self.subtitleLocalizedString = subtitleLocalizedString
        self.yesLocalizedString = yesLocalizedString
        self.noLocalizedString = noLocalizedString
        self.yesAction = yesAction
        self.noAction = noAction
        self.shouldCloseAfterYesAction = shouldCloseAfterYesAction
    }
}
