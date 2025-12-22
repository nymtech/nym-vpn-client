import SwiftUI
import Theme

public struct ActionDialogConfiguration {
    let systemIconImageName: String?
    let systemIconImageColor: Color?
    let titleLocalizedString: String?
    let subtitleLocalizedString: String?
    let yesLocalizedString: String?
    let noLocalizedString: String?
    let isYesDestructive: Bool
    let isNoDestructive: Bool
    let yesAction: (() -> Void)?
    let noAction: (() -> Void)?
    let loadingText: String?
    let shouldCloseAfterYesAction: Bool
    let verticalButtonsLayout: Bool

    public init (
        systemIconImageName: String? = nil,
        systemIconImageColor: Color? = nil,
        titleLocalizedString: String? = nil,
        subtitleLocalizedString: String? = nil,
        yesLocalizedString: String? = nil,
        noLocalizedString: String? = nil,
        isYesDestructive: Bool = false,
        isNoDestructive: Bool = false,
        yesAction: (() -> Void)? = nil,
        noAction: (() -> Void)? = nil,
        loadingText: String? = nil,
        shouldCloseAfterYesAction: Bool = true,
        verticalButtonsLayout: Bool = false
    ) {
        self.systemIconImageName = systemIconImageName
        self.systemIconImageColor = systemIconImageColor
        self.titleLocalizedString = titleLocalizedString
        self.subtitleLocalizedString = subtitleLocalizedString
        self.yesLocalizedString = yesLocalizedString
        self.noLocalizedString = noLocalizedString
        self.isYesDestructive = isYesDestructive
        self.isNoDestructive = isNoDestructive
        self.yesAction = yesAction
        self.noAction = noAction
        self.loadingText = loadingText
        self.shouldCloseAfterYesAction = shouldCloseAfterYesAction
        self.verticalButtonsLayout = verticalButtonsLayout
    }
}
