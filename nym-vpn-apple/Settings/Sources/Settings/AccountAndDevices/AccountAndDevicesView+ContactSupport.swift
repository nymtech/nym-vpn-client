import SwiftUI
import Constants
import ExternalLinkManager
import UIComponents
import Theme

// MARK: - Contact Support -
extension AccountAndDevicesView {
    func contactSupportText() -> some View {
        HStack(spacing: 0) {
            Text(contactSupportAttributedString())
                .tint(NymColor.gray1)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }
        .environment(\.openURL, OpenURLAction { url in
            if url.absoluteString == Constants.supportURL.rawValue {
                try? externalLinkManager.openExternalURL(urlString: url.absoluteString)
                return .handled
            }
            return .systemAction
        })
    }

    func contactSupportAttributedString() -> AttributedString {
        let bolt = AttributedString("⚡ ")
        var link = AttributedString("settings.account.contactSupport.link".localizedString)
        link.underlineStyle = .single
        link.foregroundColor = NymColor.primary
        link.link = URL(string: Constants.supportURL.rawValue)
        let suffix = AttributedString(" \("settings.account.contactSupport.suffix".localizedString)")
        return bolt + link + suffix
    }
}
