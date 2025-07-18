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
            }
            .padding(16)
        }
        .frame(maxWidth: .infinity, maxHeight: 68)
        .background(NymColor.elevation)
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(NymColor.error, lineWidth: 1)
        )
        .padding(24)
    }

    public init(config: GenericBannerViewConfig) {
        self.config = config
    }
}

private extension GenericBannerView {
    var titleSubtitleText: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(config.title)
                .textStyle(.Body.Large.regular)
                .foregroundStyle(NymColor.primary)

            Text(config.subtitle)
                .textStyle(.Body.Small.regular)
                .foregroundStyle(NymColor.gray1)
        }
    }

    var actionText: some View {
        VStack(spacing: 0) {
            Spacer()
            Text(config.actionTitle)
                .textStyle(.Body.Large.regular)
                .foregroundStyle(NymColor.action)
            Spacer()
        }
        .onTapGesture {
            config.action()
        }
        .accessibilityAction {
            config.action()
        }
    }
}
