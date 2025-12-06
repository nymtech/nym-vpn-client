import SwiftUI
import AppSettings
import ConnectionManager
import CountriesManagerTypes
import FeatureFlagsManager
import GatewayManager
import Theme

public struct HopButton: View {
    private let hopType: HopType
    private let buttonAction: () -> Void
    private let accessoryAction: () -> Void
    private let accessoryAccessibilityText: String

    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var gatewayManager: GatewayManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @State private var isHovered = false

    private var gatewayType: NodeType {
        switch connectionManager.connectionType {
        case .wireguard:
            .vpn
        case .mixnet5hop:
            switch hopType {
            case .entry:
                .entry
            case .exit:
                .exit
            }
        }
    }

    private var shouldShowQuic: Bool {
        hopType == .entry
        && connectionManager.connectionType == .wireguard
        && appSettings.isQuicEnabled
        && gatewayManager.containsQuic(with: connectionManager.entryGateway)
    }

    private var shouldShowStreaming: Bool {
        hopType == .exit
        && connectionManager.connectionType == .wireguard
        && gatewayManager.containsStreaming(with: connectionManager.exitRouter)
    }

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
        let gateway = gatewayManager.gateway(with: gatewayId, gatewayType: gatewayType)
        guard let location = gateway?.location
        else {
            return nameOrId(gateway: gateway)
        }

        switch hopType {
        case .entry:
            switch connectionManager.entryGateway {
            case let .country(countryCode), let .lowLatencyCountry(countryCode):
                return countrySubtitle(gateway: gateway, countryCode: countryCode, location: location)
            case .region:
                return regionSubtitle(gateway: gateway, location: location)
            case .gateway:
                return serverSubtitle(location: location, countryCode: location.twoLetterIsoCountryCode)
            case .random:
                return nil
            }
        case .exit:
            switch connectionManager.exitRouter {
            case let .country(countryCode):
                return countrySubtitle(gateway: gateway, countryCode: countryCode, location: location)
            case .gateway:
                return serverSubtitle(location: location, countryCode: location.twoLetterIsoCountryCode)
            case .region:
                return regionSubtitle(gateway: gateway, location: location)
            case .random, .address:
                return nil
            }
        }
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
                button()
                accessory()
            }
        }
        .onHover { isHovered = $0 }
    }

    public init(
        hopType: HopType,
        buttonAction: @escaping () -> Void,
        accessoryAction: @escaping () -> Void,
        accessoryAccessibilityText: String
    ) {
        self.hopType = hopType
        self.buttonAction = buttonAction
        self.accessoryAction = accessoryAction
        self.accessoryAccessibilityText = accessoryAccessibilityText
    }
}

// MARK: - Subtitle -
private extension HopButton {
    func countrySubtitle(gateway: GatewayNode?, countryCode: String, location: GatewayNodeLocation) -> String? {
        if gatewayManager.shouldDisplayRegion(with: countryCode) {
            "\(location.city), \(location.region) \(nameOrId(gateway: gateway))"
        } else {
            "\(location.city) \(nameOrId(gateway: gateway))"
        }
    }

    func regionSubtitle(gateway: GatewayNode?, location: GatewayNodeLocation) -> String? {
        "\(location.city) \(nameOrId(gateway: gateway))"
    }

    func citySubtitle(gateway: GatewayNode?, location: GatewayNodeLocation) -> String? {
        if let country = gatewayManager.localizedCountry(with: location.twoLetterIsoCountryCode) {
            "\(location.region), \(country.name) \(nameOrId(gateway: gateway))"
        } else {
            "\(location.region) \(nameOrId(gateway: gateway))"
        }
    }

    func serverSubtitle(location: GatewayNodeLocation, countryCode: String) -> String? {
        let country = gatewayManager.localizedCountry(with: countryCode)
        if gatewayManager.shouldDisplayRegion(with: countryCode) {
            return "\(location.city), \(location.region), \(country?.name ?? "")"
        } else {
            return "\(location.city), \(country?.name ?? "")"
        }
    }

    func nameOrId(gateway: GatewayNode?) -> String {
        if let name = gateway?.name {
            "(\(name))"
        } else if let identifier = gateway?.id {
            "(\(identifier))"
        } else {
            ""
        }
    }
}

private extension HopButton {
    @ViewBuilder
    func button() -> some View {
        HStack {
            if isQuickest {
                BoltImage()
                    .padding(12)
            } else if let code = hopCountryCode {
                FlagImage(countryCode: code)
                    .padding(12)
            } else {
                GenericImage(imageName: "pin")
                    .frame(width: 24, height: 24)
                    .padding(12)
            }

            titleSubtitleText(with: titleText, subtitle: subtitleText)

            Spacer()
            if shouldShowQuic {
                QuicLabel()
            } else if shouldShowStreaming {
                StreamingIcon()
            }
        }
        .onTapGesture(perform: buttonAction)
        .accessibilityElement(children: .combine)
        .accessibilityAction(.default, buttonAction)
        .accessibilityAddTraits([.isButton])
        .accessibilityLabel("\(hopType.hopLocalizedTitle) \(titleText)")
    }

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

    @ViewBuilder
    func accessory() -> some View {
        Image("arrowRight", bundle: .module)
            .resizable()
            .frame(width: 24, height: 24)
            .padding(EdgeInsets(top: 16, leading: 4, bottom: 16, trailing: 16))
            .onTapGesture(perform: accessoryAction)
            .accessibilityElement(children: .combine)
            .accessibilityAction(.default, accessoryAction)
            .accessibilityAddTraits([.isButton])
            .accessibilityLabel("\(hopType.hopLocalizedTitle) \(titleText) \(accessoryAccessibilityText)")
    }
}
