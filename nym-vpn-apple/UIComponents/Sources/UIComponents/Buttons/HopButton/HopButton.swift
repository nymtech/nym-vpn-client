import SwiftUI
import AppSettings
import ConnectionManager
import GatewayManager
import Theme

public struct HopButton: View {
    private let hopType: HopType

    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var gatewayManager: GatewayManager
    @State private var isHovered = false

    private var gatewayId: String? {
        switch hopType {
        case .entry:
            connectionManager.connectionInfoData?.entryGatewayId
        case .exit:
            connectionManager.connectionInfoData?.exitGatewayId
        }
    }

    private var titleText: String {
        switch hopType {
        case .entry:
            gatewayManager.userFriendlyTitle(with: connectionManager.entryGateway) ?? ""
        case .exit:
            gatewayManager.userFriendlyTitle(with: connectionManager.exitRouter) ?? ""
        }
    }

    private var subtitleText: String? {
        switch hopType {
        case .entry:
            guard connectionManager.entryGateway.isCountry
                    || connectionManager.entryGateway.isRegion
            else {
                return nil
            }
        case .exit:
            guard connectionManager.exitRouter.isCountry
                    || connectionManager.exitRouter.isRegion
            else {
                return nil
            }
        }
        return gatewayManager.moniker(with: gatewayId) ?? gatewayId
    }

    private var hopCountryCode: String? {
        switch hopType {
        case .entry:
            gatewayManager.countryCode(with: connectionManager.entryGateway)
        case .exit:
            gatewayManager.countryCode(with: connectionManager.exitRouter)
        }
    }

    private var isQuickest: Bool {
        switch hopType {
        case .entry:
            connectionManager.entryGateway.isQuickest
        case .exit:
            false
        }
    }

    public var body: some View {
        StrokeBorderView(
            strokeTitle: hopType.hopLocalizedTitle,
            strokeTitleLeftMargin: 30,
            isHovered: $isHovered
        ) {
            HStack {
                if isQuickest {
                    BoltImage()
                        .padding(12)
                } else if let code = hopCountryCode {
                    FlagImage(countryCode: code)
                        .padding(12)
                }

                titleSubtitleText(with: titleText, subtitle: subtitleText)

                Spacer()
                Image("arrowRight", bundle: .module)
                    .resizable()
                    .frame(width: 24, height: 24)
                    .padding(16)
            }
        }
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits([.isButton])
        .accessibilityLabel("\(hopType.hopLocalizedTitle) \(titleText)")
        .onHover { isHovered = $0 }
    }

    public init(hopType: HopType) {
        self.hopType = hopType
    }
}

private extension HopButton {
    @ViewBuilder
    func titleSubtitleText(with text: String, subtitle: String?) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(text)
                .lineLimit(1)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)

            if let subtitle, !subtitle.isEmpty {
                Text(subtitle)
                    .lineLimit(1)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Small.regular)
            }
        }
        .animation(.default, value: subtitle)
    }
}
