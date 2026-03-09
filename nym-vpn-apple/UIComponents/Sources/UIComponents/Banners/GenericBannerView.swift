import SwiftUI
import Theme

public struct GenericBannerView: View {
    private let config: GenericBannerViewConfig

    public var body: some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                titleSubtitleText
                Spacer()
                actionText
                closeButton
            }
            .padding(16)
        }
        .frame(maxWidth: .infinity, maxHeight: 68)
        .background(NymColor.elevation)
        .cornerRadius(8)
        .padding(.horizontal, 16)
    }

    public init(config: GenericBannerViewConfig) {
        self.config = config
    }
}

private extension GenericBannerView {
    var titleSubtitleText: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(config.title)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)

            Text(config.subtitle)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.gray1)
        }
    }

    var actionText: some View {
        Text(config.actionTitle)
            .textStyle(.Body.Medium.regular)
            .foregroundStyle(NymColor.accent)
            .multilineTextAlignment(.center)
            .fixedSize(horizontal: false, vertical: true)
            .frame(maxWidth: 120)
            .onTapGesture {
                config.action()
            }
            .accessibilityAction {
                config.action()
            }
    }

    @ViewBuilder
    var closeButton: some View {
        if let closeAction = config.closeAction {
            GenericImage(systemImageName: "xmark")
                .foregroundStyle(NymColor.primary)
                .frame(width: 12, height: 12)
                .padding(8)
                .contentShape(Rectangle())
                .onTapGesture {
                    closeAction()
                }
        }
    }
}
