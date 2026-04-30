#if os(iOS)
import SwiftUI

extension View {
    public func inAppSafari(using manager: ExternalLinkManager) -> some View {
        modifier(InAppSafariModifier(manager: manager))
    }
}

private struct InAppSafariModifier: ViewModifier {
    @ObservedObject var manager: ExternalLinkManager

    func body(content: Content) -> some View {
        content
            .sheet(item: $manager.inAppSafariURL) { identifier in
                SafariView(url: identifier.url)
                    .ignoresSafeArea()
            }
    }
}
#endif
