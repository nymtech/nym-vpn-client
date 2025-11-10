import SwiftUI
import Constants
import ExternalLinkManager
import UIComponents
import Theme

struct LocationInfoView: View {
    @EnvironmentObject private var externalLinkManager: ExternalLinkManager
    @State private var isContinueReadingLinkHovered = false

    private let viewModel: LocationInfoViewModel

    init(viewModel: LocationInfoViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            HStack {
                Spacer()
                    .frame(width: 40)

                VStack {
                    icon()
                    title()
                    streamingSectionTitle()
                    streamingSubtitle()
                    Spacer()
                        .frame(height: 16)
                    locationAccuracySectionTitle()
                    locationAccuracySubtitle()
                    okButton()
                }
                .padding(.horizontal, 24)
                .background(NymColor.elevation)
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

        Image(systemName: viewModel.infoIconImageName)
            .frame(width: 24, height: 24)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text(viewModel.titleLocalizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)

        Spacer()
            .frame(height: 16)
    }

    func streamingSectionTitle() -> some View {
        HStack(spacing: 0) {
            GenericImage(systemImageName: "play.rectangle")
                .frame(width: 16, height: 16)
                .foregroundStyle(NymColor.primary)
            Spacer()
                .frame(width: 8)
            Text("locationModal.streaming".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
                .frame(height: 8)
        }
    }

    func streamingSubtitle() -> some View {
        HStack(spacing: 0) {
            Text(streamingAttributtedString())
                .tint(NymColor.gray1)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)

            Spacer()
        }
        .environment(\.openURL, OpenURLAction { url in
            if url == URL(string: Constants.streamingServicesURL.rawValue) {
                externalLinkManager.openExternalURL(url)
                return .handled
            }
            return .systemAction
        })
    }

    func streamingAttributtedString() -> AttributedString {
        var first = AttributedString("locationModal.streaming.subtitle1".localizedString)
        let second = AttributedString("locationModal.streaming.subtitle2".localizedString)
        first.underlineStyle = .single
        first.foregroundColor = NymColor.primary
        first.link = URL(string: Constants.streamingServicesURL.rawValue)
        return first + AttributedString(" ") + second
    }

    func locationAccuracySectionTitle() -> some View {
        HStack(spacing: 0) {
            GenericImage(imageName: "pin")
                .frame(width: 16, height: 16)
                .foregroundStyle(NymColor.primary)
            Spacer()
                .frame(width: 8)
            Text("locationModal.streaming".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
                .frame(height: 8)
        }
    }

    func locationAccuracySubtitle() -> some View {
        HStack(spacing: 0) {
            Text(locationAccuracyattributedString())
                .tint(NymColor.gray1)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)

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
        second.foregroundColor = NymColor.primary
        second.link = URL(string: Constants.locationAccuracyURL.rawValue)
        return first + AttributedString(" ") + second + AttributedString(" ") + third
    }

    @ViewBuilder
    func okButton() -> some View {
        GenericButton(title: viewModel.okLocalizedString)
            .padding(.vertical, 24)
            .onTapGesture {
                viewModel.isDisplayed.toggle()
            }
    }
}
