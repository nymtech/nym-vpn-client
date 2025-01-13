import SwiftUI
import ExternalLinkManager
import Theme

final class ModeSelectionInfoViewModel {
    let externalLinkManager: ExternalLinkManager
    let infoIconImageName = "info.circle"
    let titleLocalizedString = "modeSelectionInfo.title".localizedString
    let anonymousImageName = "anonymous"
    let anonymousTitleLocalizedString = "5hopMixnetTitle".localizedString
    let anonymousDescriptionLocalizedString = "modeSelectionInfo.anonymousDescription".localizedString
    let fastImageName = "fast"
    let fastTitleLocalizedString = "2hopMixnetTitle".localizedString
    let fastDescriptionLocalizedString = "modeSelectionInfo.fastDescription".localizedString
    let continueReadingLocalizedString = "modeSelectionInfo.continueReading".localizedString
    let continueReadingLinkImageName = "export"
    let continueReadingURLString = "https://support.nym.com/hc/articles/24326365096721-What-s-the-difference-between-Fast-and-Anonymous-mode"
    let okLocalizedString = "ok".localizedString

    @Binding var isDisplayed: Bool

    init(externalLinkManager: ExternalLinkManager, isDisplayed: Binding<Bool>) {
        self.externalLinkManager = externalLinkManager
        _isDisplayed = isDisplayed
    }

    func openContinueReading() {
        // TODO: log error
        try? externalLinkManager.openExternalURL(urlString: continueReadingURLString)
    }
}
