import SwiftUI
import ExternalLinkManager
import Theme

final class LocationInfoViewModel {
    let externalLinkManager: ExternalLinkManager
    let infoIconImageName = "info.circle"
    let titleLocalizedString = "locationModal.title"
    let messageLocalizedString = "locationModal.message"
    let readMoreLocalizedString = "locationModal.readMore"
    let readMoreLinkImageName = "export"
    let readMoreURLString = "https://support.nym.com/hc/articles/26448676449297-How-is-server-location-determined-by-NymVPN"
    let okLocalizedString = "ok"

    @Binding var isDisplayed: Bool

    init(externalLinkManager: ExternalLinkManager, isDisplayed: Binding<Bool>) {
        self.externalLinkManager = externalLinkManager
        _isDisplayed = isDisplayed
    }

    func openContinueReading() {
        // TODO: log error
        try? externalLinkManager.openExternalURL(urlString: readMoreURLString)
    }
}
