import SwiftUI
import ExternalLinkManager
import Theme

final class ModeSelectionInfoViewModel {
    let externalLinkManager: ExternalLinkManager
    let infoIconImageName = "info.circle"
    let titleLocalizedString = "modeSelectionInfo.title"
    let anonymousImageName = "anonymous"
    let anonymousTitleLocalizedString = "5hopMixnetTitle"
    let anonymousDescriptionLocalizedString = "modeSelectionInfo.anonymousDescription"
    let fastImageName = "fast"
    let fastTitleLocalizedString = "2hopMixnetTitle"
    let fastDescriptionLocalizedString = "modeSelectionInfo.fastDescription"
    let continueReadingLocalizedString = "modeSelectionInfo.continueReading"
    let continueReadingLinkImageName = "export"
    let continueReadingURLString = "https://support.nym.com/hc/articles/24326365096721-What-s-the-difference-between-Fast-and-Anonymous-mode"
    let okLocalizedString = "ok"

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
