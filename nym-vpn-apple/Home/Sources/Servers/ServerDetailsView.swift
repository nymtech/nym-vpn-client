import SwiftUI
import AppSettings
import Constants
import ConnectionManager
import ConnectionTypes
import Device
import ExternalLinkManager
import FeatureFlagsManager
import GatewayManager
import ImpactGenerator
import Routes
import Settings
import Theme
import UIComponents

public struct ServerDetailsView: View {
    private let externalLinkManager: ExternalLinkManager
    private let gateway: GatewayNode
    private let hopType: HopType
    @Binding private var path: NavigationPath
    @EnvironmentObject private var appSettings: AppSettings
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var gatewayManager: GatewayManager
    @EnvironmentObject private var featureFlagsManager: FeatureFlagsManager
    @State private var messageOverlayText: String?
    @State private var displayMessageOverlay = false

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            ScrollView {
                Spacer()
                    .frame(height: 16)
                if !isCurrentlyConnectedHopGateway {
                    selectServerButton()
                        .padding(.horizontal, 16)
                    Spacer()
                        .frame(height: 24)
                }
                scrollViewContent()
            }
            .scrollIndicators(.never)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.surfaceBg
                .ignoresSafeArea()
        }
    }

    public init(
        path: Binding<NavigationPath>,
        gateway: GatewayNode,
        hopType: HopType,
        externalLinkManager: ExternalLinkManager
    ) {
        _path = path
        self.gateway = gateway
        self.hopType = hopType
        self.externalLinkManager = externalLinkManager
    }
}

// MARK: - Views -
private extension ServerDetailsView {
    func navbar() -> some View {
        CustomNavBar(
            title: "gatewayInfo.serverDetails".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func scrollViewContent() -> some View {
        VStack(spacing: 0) {
            identityCard()
            Spacer()
                .frame(height: 24)
            capabilitiesSection()
            Spacer()
                .frame(height: 24)
            performanceSection()
            Spacer()
                .frame(height: 24)
            ipSection()
            Spacer()
                .frame(height: 24)
            serverInfoSection()
            Spacer()
                .frame(height: 24)
            missingInfoText()
            Spacer()
                .frame(height: 24)
            explorer()
            Spacer()
                .frame(height: 24)
        }
        .padding(.horizontal, 16)
    }

    func sectionHeader(with title: String) -> some View {
        HStack(spacing: 0) {
            Text(title)
                .foregroundStyle(Color.Nym.brandPrimary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
        .padding(.bottom, 8)
    }

    func identityCard() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 8) {
                FlagImage(countryCode: gateway.location?.twoLetterIsoCountryCode, width: 24, height: 24)
                Text(gateway.name ?? gateway.id)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyLarge)
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.top, 16)

            Rectangle()
                .foregroundColor(Color.Nym.gray2)
                .frame(height: 1)
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)

            VStack(alignment: .leading, spacing: 12) {
                Text(locationTitle())
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyDefault)
                    .underline()
                if let description = gateway.description, !description.isEmpty {
                    Text(description)
                        .multilineTextAlignment(.leading)
                        .foregroundStyle(Color.Nym.textSecondary)
                        .nymTextStyle(.bodyDefault)
                }
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 16)
        }
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color.Nym.surfaceElev)
        .cornerRadius(12)
    }

    func missingInfoText() -> some View {
        HStack(spacing: 0) {
            Text(missingInfoAttributedString() ?? "")
                .tint(Color.Nym.statusInfo)
                .foregroundStyle(Color.Nym.statusInfo)
                .nymTextStyle(.bodyDefault)
                .underline()
            exportImage()
                .padding(.horizontal, 8)
            Spacer()
        }
        .accessibilityAction {
            openExternalLink(with: Constants.serverLocationURL.rawValue)
        }
        .onTapGesture {
            openExternalLink(with: Constants.serverLocationURL.rawValue)
        }
    }

    func missingInfoAttributedString() -> AttributedString? {
        try? AttributedString(markdown: "[\("gatewayInfo.missingInfo".localizedString)](\(Constants.serverLocationURL.rawValue))")
    }

    func explorer() -> some View {
        HStack(spacing: 0) {
            Text(explorerAttributedString() ?? "")
                .tint(Color.Nym.textSecondary)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            exportImage()
                .padding(.horizontal, 8)
            Spacer()
        }.accessibilityAction {
            openExternalLink(with: "\(Constants.explorerURL.rawValue)\(gateway.id)")
        }
        .onTapGesture {
            openExternalLink(with: "\(Constants.explorerURL.rawValue)\(gateway.id)")
        }
    }

    func explorerAttributedString() -> AttributedString? {
        let markdown = "\("gatewayInfo.moreDetailsIn".localizedString) [\("gatewayInfo.networkExplorer".localizedString)](\(Constants.explorerURL.rawValue)\(gateway.id))"

        guard var explorerMarkdownString = try? AttributedString(markdown: markdown) else { return nil }
        for run in explorerMarkdownString.runs where run.link != nil {
            explorerMarkdownString[run.range].underlineStyle = .single
            explorerMarkdownString[run.range].foregroundColor = Color.Nym.statusInfo
        }
        return explorerMarkdownString
    }

    func selectServerButton() -> some View {
        GenericButton(title: "gatewayInfo.selectServer".localizedString)
            .accessibilityAction {
                selectServer()
            }
            .onTapGesture {
                selectServer()
            }
    }

    func separatorLine() -> some View {
        Rectangle()
            .foregroundColor(Color.Nym.gray2)
            .frame(height: 1)
            .padding(.vertical, 12)
    }

    func rowTitle(with title: String) -> some View {
        Text(title)
            .foregroundStyle(Color.Nym.textSecondary)
            .nymTextStyle(.bodyDefault)
    }

    func rowSubtite(with subtitle: String) -> some View {
        Text(subtitle)
            .foregroundStyle(Color.Nym.textPrimary)
            .nymTextStyle(.bodyDefault)
    }

    func exportImage() -> some View {
        GenericImage(imageName: "export")
            .frame(width: 16, height: 16)
            .foregroundStyle(Color.Nym.statusInfo)
    }
}

private extension ServerDetailsView {
    func capabilitiesSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("gatewayInfo.section.serverFeatures".localizedString)
                .foregroundStyle(Color.Nym.brandPrimary)
                .nymTextStyle(.bodyDefault)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.bottom, 16)
            advancedPrivacyRow()
            separatorLine()
            streamingAndContentRow()
            separatorLine()
            postQuantumRow()
            separatorLine()
            bridges()
            Spacer()
                .frame(height: 16)
            bridgesInfo()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color.Nym.surfaceElev)
        .cornerRadius(12)
    }

    func postQuantumRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.postQuantumSecureKeys".localizedString)
            Spacer()
            GenericImage(imageName: "quantum")
                .frame(width: 15, height: 15)
                .foregroundStyle(Color.Nym.brandPrimary)
                .padding(.trailing, 6)
            rowSubtite(with: "gatewayInfo.lewesProtocol".localizedString)
        }
    }

    func advancedPrivacyRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.advancedPrivacy".localizedString)
            Spacer()
            GenericImage(imageName: "mixnet26")
                .frame(width: 15, height: 15)
                .foregroundStyle(Color.Nym.brandPrimary)
                .padding(.trailing, 6)
            rowSubtite(with: "gatewayInfo.mixnet".localizedString)
        }
    }

    func streamingAndContentRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.streamingAndIp".localizedString)
            Spacer()
            switch gateway.location?.asn?.type {
            case .residential:
                GenericImage(imageName: "smartDisplay")
                    .frame(width: 15, height: 15)
                    .foregroundStyle(Color.Nym.statusInfo)
                    .padding(.trailing, 6)
                rowSubtite(with: "gatewayInfo.residentialIp".localizedString)
            case .other, nil:
                GenericImage(imageName: "datacenter")
                    .frame(width: 15, height: 15)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .padding(.trailing, 6)
                rowSubtite(with: "gatewayInfo.datacenter".localizedString)
            }
        }
    }

    func bridges() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.antiCensorship".localizedString)
            Spacer()

            if gateway.isQuicAvailable {
                GenericImage(systemImageName: "shippingbox")
                    .frame(width: 15, height: 15)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .padding(.trailing, 6)
            } else {
                GenericImage(systemImageName: "circle.fill")
                    .frame(width: 10, height: 10)
                    .foregroundStyle(Color.Nym.statusWarning)
                    .padding(.trailing, 6)
            }

            rowSubtite(
                with: gateway.isQuicAvailable
                ? "gatewayInfo.quicProtocol".localizedString
                : "gatewayInfo.standardProtocol".localizedString
            )
        }
    }

    func bridgesInfo() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(enableQuicText())
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodySmall)
        }
        .environment(\.openURL, OpenURLAction { url in
            if url == URL(string: "app://enable-quic") {
                path.append(HomeLink.settings)
                path.append(SettingLink.censorship)
                return .handled
            }
            return .systemAction
        })
    }

    func enableQuicText() -> AttributedString {
        let first = "gatewayInfo.enableQuic1".localizedString
        let second = "gatewayInfo.enableQuic2".localizedString
        var firstAttr = AttributedString(first)
        firstAttr.underlineStyle = .single
        firstAttr.foregroundColor = Color.Nym.statusInfo
        firstAttr.link = URL(string: "app://enable-quic")
        let secondAttr = AttributedString(second)
        return firstAttr + AttributedString(" ") + secondAttr
    }
}

// MARK: - Performance section -
private extension ServerDetailsView {
    func performanceSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("gatewayInfo.section.performanceMetrics".localizedString)
                .foregroundStyle(Color.Nym.brandPrimary)
                .nymTextStyle(.bodyDefault)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.bottom, 16)
            overAllPerformanceRow()
            separatorLine()
            serverLoadRow()
            separatorLine()
            uptimeRow()
            Spacer()
                .frame(height: 12)
            Text(formattedLastUpdate())
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodySmall)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color.Nym.surfaceElev)
        .cornerRadius(12)
    }

    func overAllPerformanceRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.overallPerformance".localizedString)
            Spacer()
            GenericImage(imageName: gateway.performance?.score.imageName)
                .frame(width: 16, height: 16)
                .padding(8)
            Text(
                gateway.performance?.score.localizedKey.localizedString
                ?? GatewayNodeScore.noScore.localizedKey.localizedString
            )
            .foregroundStyle(
                scoreOverallPerformanceImageColor(
                    with: gateway.performance?.score ?? GatewayNodeScore.noScore
                )
            )
            .nymTextStyle(.bodyDefault)
        }
    }

    func serverLoadRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.serverLoad".localizedString)
            Spacer()
            Text(
                gateway.performance?.load.localizedKey.localizedString
                ?? GatewayNodeScore.noScore.localizedKey.localizedString
            )
            .foregroundStyle(
                scoreLoadImageColor(
                    with: gateway.performance?.load ?? GatewayNodeScore.noScore
                )
            )
            .nymTextStyle(.bodyDefault)
        }
    }

    func uptimeRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.uptime".localizedString)
            Spacer()
            rowSubtite(with: formattedUptime())
        }
    }
}

// MARK: - Server info -
private extension ServerDetailsView {
    func serverInfoSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("gatewayInfo.section.buildInformation".localizedString)
                .foregroundStyle(Color.Nym.brandPrimary)
                .nymTextStyle(.bodyDefault)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.bottom, 16)
            buildVersionRow()
            separatorLine()
            identityRow()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color.Nym.surfaceElev)
        .cornerRadius(12)
    }

    func buildVersionRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.buildVersion".localizedString)
            Spacer()
            rowSubtite(with: gateway.buildVersion ?? "noScore".localizedString)
        }
    }

    func identityRow() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            rowTitle(with: "\("gatewayInfo.identityKey".localizedString):")
            Spacer()
                .frame(height: 8)
            HStack(spacing: 0) {
                Text(gateway.id)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyDefault)
                Spacer()
                    .frame(width: 16)
                GenericImage(imageName: "copy")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .padding(10)
                    .contentShape(Rectangle())
                    .onTapGesture {
                        copyToPasteboard()
                    }
                    .accessibilityLabel("gatewayInfo.copyIdentityKey".localizedString)
                    .accessibilityHint("accessibility.doubleTap.copy".localizedString)
                    .accessibilityAddTraits([.isButton])
                    .accessibilityAction {
                        copyToPasteboard()
                    }
            }
        }
        .overlay {
            if displayMessageOverlay, let messageOverlayText {
                HStack {
                    Spacer()
                    Text(messageOverlayText)
                        .padding(8)
                        .background(Color.Nym.surfaceBg)
                        .foregroundColor(Color.Nym.textSecondary)
                        .cornerRadius(12)
                        .transition(.opacity)
                        .padding(.trailing, 0)
                }
                .animation(.easeInOut, value: displayMessageOverlay)
            }
        }
    }
}

// MARK: - IP section -
private extension ServerDetailsView {
    func ipSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("gatewayInfo.section.connectionDetails".localizedString)
                .foregroundStyle(Color.Nym.brandPrimary)
                .nymTextStyle(.bodyDefault)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.bottom, 16)
            ipv4Rows()
            ipv6Rows()
            asnRow()
            separatorLine()
            asnNameRow()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(Color.Nym.surfaceElev)
        .cornerRadius(12)
    }

    @ViewBuilder
    func ipv4Rows() -> some View {
        if !gateway.ipv4s.isEmpty {
            HStack(spacing: 0) {
                rowTitle(with: "gatewayInfo.ipv4".localizedString)
                Spacer()
                VStack(spacing: 0) {
                    ForEach(gateway.ipv4s, id: \.self) { ip in
                        ipRowCell(with: ip)
                    }
                }
            }
            separatorLine()
        }
    }

    @ViewBuilder
    func ipv6Rows() -> some View {
        if !gateway.ipv6s.isEmpty {
            HStack(spacing: 0) {
                rowTitle(with: "gatewayInfo.ipv6".localizedString)
                Spacer()
                VStack(spacing: 0) {
                    ForEach(gateway.ipv6s, id: \.self) { ip in
                        ipRowCell(with: ip)
                    }
                }
            }
            separatorLine()
        }
    }

    func ipRowCell(with ip: String) -> some View {
        HStack(spacing: 8) {
            Text(ip)
                .foregroundStyle(Color.Nym.statusInfo)
                .nymTextStyle(.bodyDefault)
                .underline()
            exportImage()
        }
        .accessibilityAction {
            openExternalLink(with: "\(Constants.ipInfoURL.rawValue)\(ip)")
        }
        .onTapGesture {
            openExternalLink(with: "\(Constants.ipInfoURL.rawValue)\(ip)")
        }
    }

    @ViewBuilder
    func asnRow() -> some View {
        if let asn = gateway.location?.asn {
            HStack(spacing: 0) {
                rowTitle(with: "gatewayInfo.asn".localizedString)
                Spacer()
                rowSubtite(with: asn.asn)
            }
        }
    }

    @ViewBuilder
    func asnNameRow() -> some View {
        if let asn = gateway.location?.asn {
            HStack(spacing: 0) {
                rowTitle(with: "gatewayInfo.asnName".localizedString)
                Spacer()
                rowSubtite(with: asn.asnName)
            }
        }
    }
}

// MARK: - Actions
private extension ServerDetailsView {
    var isCurrentlyConnectedHopGateway: Bool {
        guard let info = connectionManager.connectionInfoData else { return false }
        switch hopType {
        case .entry:
            return info.entryGatewayId == gateway.id
        case .exit:
            return info.exitGatewayId == gateway.id
        }
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func copyToPasteboard() {
#if os(iOS)
        UIPasteboard.general.string = gateway.id
        ImpactGenerator.shared.impact()
#elseif os(macOS)
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(gateway.id, forType: .string)
#endif
        withAnimation {
            messageOverlayText = "settings.copiedToPasteboard".localizedString
            displayMessageOverlay = true
            shcheduleMessageOverlayDismissal()
        }
    }

    func openExternalLink(with link: String) {
        #if os(iOS)
        ImpactGenerator.shared.impact()
        #endif
        try? externalLinkManager.openExternalURL(urlString: link)
    }

    func selectServer() {
#if os(iOS)
        ImpactGenerator.shared.impact()
#endif
        switch hopType {
        case .entry:
            connectionManager.setEntryGateway(.gateway(gateway.id))
        case .exit:
            connectionManager.applyExplicitExit(.gateway(gateway.id))
        }
        path = .init()
    }
}

// MARK: - Helpers -
private extension ServerDetailsView {
    // TODO: check if working
    func locationTitle() -> String {
        let parts = [
            gateway.location?.city,
            gateway.location?.region,
            gatewayManager.localizedCountry(with: gateway.location?.twoLetterIsoCountryCode)?.name
        ]
        .compactMap { $0 }
        .filter { !$0.isEmpty }
        return parts.joined(separator: ", ")
    }

    func scoreOverallPerformanceImageColor(with score: GatewayNodeScore) -> Color {
        switch score {
        case .noScore:
            Color.Nym.textSecondary
        case .low:
            Color.Nym.statusError
        case .medium:
            Color.Nym.statusWarning
        case .high:
            Color.Nym.brandPrimary
        case .offline:
            Color.Nym.textSecondary
        }
    }

    func scoreLoadImageColor(with score: GatewayNodeScore) -> Color {
        switch score {
        case .noScore:
            Color.Nym.textSecondary
        case .low:
            Color.Nym.brandPrimary
        case .medium:
            Color.Nym.statusWarning
        case .high:
            Color.Nym.statusError
        case .offline:
            Color.Nym.textSecondary
        }
    }

    func formattedUptime() -> String {
        let formatter = NumberFormatter()
        formatter.numberStyle = .percent
        formatter.maximumFractionDigits = 0
        formatter.locale = .current
        guard let uptime = gateway.performance?.uptime,
              let number = formatter.string(from: NSNumber(value: uptime))
        else {
            return "noScore".localizedString
        }
        return number
    }

    func formattedLastUpdate() -> String {
        guard let date = gateway.performance?.lastUpdated
        else {
            return "noScore".localizedString
        }

        let formatter = RelativeDateTimeFormatter()
        formatter.unitsStyle = .full

        let relativeString = formatter.localizedString(for: date, relativeTo: Date())
        return "\("gatewayInfo.lastUpdate".localizedString): \(relativeString)."
    }

    func shcheduleMessageOverlayDismissal() {
        Task { @MainActor in
            try? await Task.sleep(for: .seconds(3))
            displayMessageOverlay = false
        }
    }
}
