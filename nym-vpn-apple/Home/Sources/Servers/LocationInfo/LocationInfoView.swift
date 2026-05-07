import SwiftUI
import Constants
import ExternalLinkManager
import UIComponents
import Theme

struct LocationInfoView: View {
    private let type: HopType
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @State private var isContinueReadingLinkHovered = false
    @Binding private var isDisplayed: Bool

    init(type: HopType, isDisplayed: Binding<Bool>) {
        _isDisplayed = isDisplayed
        self.type = type
    }

    var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(Color.Nym.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            HStack {
                Spacer()
                    .frame(width: 40)

                VStack {
                    icon()
                    title()
                    streamingOrQuicSectionTitle()
                    streamingOrQuicSubtitle()
                    Spacer()
                        .frame(height: 16)
                    locationAccuracySectionTitle()
                    locationAccuracySubtitle()
                    okButton()
                }
                .padding(.horizontal, 24)
                .background(Color.Nym.backgroundCard)
                .cornerRadius(16)

                Spacer()
                    .frame(width: 40)
            }
        }
        .edgesIgnoringSafeArea(.all)
    }
}

private extension LocationInfoView {
    @ViewBuilder
    func icon() -> some View {
        Spacer()
            .frame(height: 24)

        Image(systemName: "info.circle")
            .frame(width: 24, height: 24)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text(titleText())
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)

        Spacer()
            .frame(height: 16)
    }

    func streamingOrQuicSectionTitle() -> some View {
        HStack(spacing: 0) {
            GenericImage(systemImageName: streamingOrQuickImageName())
                .frame(width: 16, height: 16)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
                .frame(width: 8)
            Text(streamingOrQuicTitle())
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }
    }

    func streamingOrQuickImageName() -> String {
        switch type {
        case .entry:
            "shippingbox.fill"
        case .exit:
            "play.rectangle"
        }
    }

    func streamingOrQuicTitle() -> String {
        switch type {
        case .entry:
            "locationModal.quic".localizedString
        case .exit:
            "locationModal.streaming".localizedString
        }
    }

    func streamingOrQuicSubtitle() -> some View {
        HStack(spacing: 0) {
            Text(quicOrStreamingAttributtedString())
                .tint(Color.Nym.textSecondary)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)

            Spacer()
        }
        .environment(\.openURL, OpenURLAction { url in
            if url == URL(string: Constants.streamingServicesURL.rawValue)
                || url == URL(string: Constants.quicURL.rawValue) {
                externalLinkManager.openExternalURL(url)
                return .handled
            }
            return .systemAction
        })
    }

    func quicOrStreamingAttributtedString() -> AttributedString {
        switch type {
        case .entry:
            quicAttributtedString()
        case .exit:
            streamingAttributtedString()
        }
    }

    func quicAttributtedString() -> AttributedString {
        let first = AttributedString("locationModal.quic.subtitle1".localizedString)
        var second = AttributedString("locationModal.quic.subtitle2".localizedString)
        let third = AttributedString("locationModal.quic.subtitle3".localizedString)
        second.underlineStyle = .single
        second.foregroundColor = Color.Nym.textPrimary
        second.link = URL(string: Constants.quicURL.rawValue)
        return first + AttributedString(" ") + second + AttributedString(" ") + third
    }

    func streamingAttributtedString() -> AttributedString {
        var first = AttributedString("locationModal.streaming.subtitle1".localizedString)
        let second = AttributedString("locationModal.streaming.subtitle2".localizedString)
        first.underlineStyle = .single
        first.foregroundColor = Color.Nym.textPrimary
        first.link = URL(string: Constants.streamingServicesURL.rawValue)
        return first + AttributedString(" ") + second
    }

    func locationAccuracySectionTitle() -> some View {
        HStack(spacing: 0) {
            GenericImage(imageName: "pin")
                .frame(width: 16, height: 16)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
                .frame(width: 8)
            Text("locationModal.locationAccuracy".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }
    }

    func locationAccuracySubtitle() -> some View {
        HStack(spacing: 0) {
            Text(locationAccuracyattributedString())
                .tint(Color.Nym.textSecondary)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)

            Spacer()
        }
        .environment(\.openURL, OpenURLAction { url in
            if url == URL(string: Constants.locationAccuracyURL.rawValue) {
                externalLinkManager.openExternalURL(url)
                return .handled
            }
            return .systemAction
        })
    }

    func locationAccuracyattributedString() -> AttributedString {
        let first = AttributedString("locationModal.accuracy.subtitle1".localizedString)
        var second = AttributedString("locationModal.accuracy.subtitle2".localizedString)
        let third = AttributedString("locationModal.accuracy.subtitle3".localizedString)
        second.underlineStyle = .single
        second.foregroundColor = Color.Nym.textPrimary
        second.link = URL(string: Constants.locationAccuracyURL.rawValue)
        return first + AttributedString(" ") + second + AttributedString(" ") + third
    }

    @ViewBuilder
    func okButton() -> some View {
        GenericButton(title: "ok".localizedString)
            .padding(.vertical, 24)
            .onTapGesture {
                isDisplayed.toggle()
            }
    }
}

private extension LocationInfoView {
    func titleText() -> String {
        switch type {
        case .exit:
            "locationModal.exit.title".localizedString
        case .entry:
            "locationModal.entry.title".localizedString
        }
    }
}
