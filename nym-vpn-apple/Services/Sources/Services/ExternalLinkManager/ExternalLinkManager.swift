#if os(iOS)
import UIKit
#endif

#if os(macOS)
import AppKit
#endif

import Constants

@MainActor public final class ExternalLinkManager: ObservableObject {
    public static let shared = ExternalLinkManager()

#if os(iOS)
    @Published public var inAppSafariURL: InAppSafariURL?

    public func openExternalURL(urlString: String?) throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        openExternalURL(url)
    }

    public func openExternalURL(_ url: URL) {
        UIApplication.shared.open(url)
    }

    public func openInAppBrowser(urlString: String?) throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        openInAppBrowser(url)
    }

    public func openInAppBrowser(_ url: URL) {
        guard let scheme = url.scheme?.lowercased(),
              scheme == "https"
        else {
            openExternalURL(url)
            return
        }
        inAppSafariURL = InAppSafariURL(url: url)
    }
#endif

#if os(macOS)
    public func openExternalURL(urlString: String?) throws {
        guard let urlString, let url = URL(string: urlString)
        else {
            throw GeneralNymError.invalidUrl
        }
        openExternalURL(url)
    }

    public func openExternalURL(_ url: URL) {
        NSWorkspace.shared.open(url)
    }
#endif
}
