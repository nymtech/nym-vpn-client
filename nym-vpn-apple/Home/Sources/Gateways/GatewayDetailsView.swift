import SwiftUI
import Constants
import ConnectionManager
import CountriesManager
import CountriesManagerTypes
import ExternalLinkManager
#if os(iOS)
import ImpactGenerator
#endif
import Theme
import UIComponents

public struct GatewayDetailsView: View {
    private let externalLinkManager: ExternalLinkManager
    private let gateway: GatewayNode
    private let hopType: HopType
    @Binding private var path: NavigationPath
    @EnvironmentObject private var connectionManager: ConnectionManager
    @EnvironmentObject private var countriesManager: CountriesManager
    @State private var messageOverlayText: String?
    @State private var displayMessageOverlay = false

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            ScrollView {
                Spacer()
                    .frame(height: 24)
                VStack(spacing: 0) {
                    serverTitle()
                    Spacer()
                        .frame(height: 16)
                    location()
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
            selectServerSection()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init(
        path: Binding<NavigationPath>,
        gateway: GatewayNode,
        hopType: HopType,
        externalLinkManager: ExternalLinkManager = .shared
    ) {
        _path = path
        self.gateway = gateway
        self.hopType = hopType
        self.externalLinkManager = externalLinkManager
    }
}

// MARK: - Views -
private extension GatewayDetailsView {
    func navbar() -> some View {
        CustomNavBar(
            title: "gatewayInfo.serverDetails".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    func serverTitle() -> some View {
        HStack {
            Text(gateway.moniker ?? gateway.id)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Headline.Medium.regular)
            Spacer()
        }
    }

    func location() -> some View {
        HStack(spacing: 0) {
            FlagImage(countryCode: gateway.location?.twoLetterIsoCountryCode, width: 16, height: 16)
            Spacer()
                .frame(width: 8)
            Text(locationTitle())
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
            Spacer()
        }
    }

    func missingInfoText() -> some View {
        HStack(spacing: 0) {
            Text(missingInfoAttributedString() ?? "")
                .tint(NymColor.gray1)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
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
                .tint(NymColor.gray1)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
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
        }
        return explorerMarkdownString
    }

    func selectServerSection() -> some View {
        VStack(alignment: .center, spacing: 0) {
            Rectangle()
                .foregroundColor(NymColor.gray2)
                .frame(height: 1)
            GenericButton(title: "gatewayInfo.selectServer".localizedString)
                .padding(EdgeInsets(top: 24, leading: 16, bottom: 0, trailing: 16))
#if os(macOS)
            Spacer()
                .frame(height: 24)
#endif
        }
        .background(NymColor.elevation)
        .frame(maxWidth: .infinity, alignment: .center)
        .accessibilityAction {
            selectServer()
        }
        .onTapGesture {
            selectServer()
        }
    }

    func separatorLine() -> some View {
        Rectangle()
            .foregroundColor(NymColor.gray2)
            .frame(height: 1)
            .padding(.vertical, 12)
    }

    func rowTitle(with title: String) -> some View {
        Text(title)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Medium.regular)
    }

    func rowSubtite(with subtitle: String) -> some View {
        Text(subtitle)
            .foregroundStyle(NymColor.primary)
            .textStyle(.Body.Medium.regular)
    }

    func exportImage() -> some View {
        GenericImage(imageName: "export")
            .frame(width: 16, height: 16)
            .foregroundStyle(NymColor.primary)
    }
}

private extension GatewayDetailsView {
    func capabilitiesSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            advancedPrivacyRow()
            separatorLine()
            streamingAndContentRow()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(NymColor.elevation)
        .cornerRadius(8)
    }

    func advancedPrivacyRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.advancedPrivacy".localizedString)
            Spacer()
            GenericImage(systemImageName: "checkmark")
                .frame(width: 10, height: 10)
                .foregroundStyle(NymColor.accent)
                .padding(.horizontal, 8)
            rowSubtite(with: "gatewayInfo.advancedPrivacySubtitle".localizedString)
        }
    }

    func streamingAndContentRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.streamingAndContent".localizedString)
            Spacer()
            switch gateway.location?.asn?.type {
            case .residential:
                GenericImage(systemImageName: "checkmark")
                    .frame(width: 10, height: 10)
                    .foregroundStyle(NymColor.accent)
                    .padding(.horizontal, 8)
                rowSubtite(with: "gatewayInfo.residentialIp".localizedString)
            case .other:
                GenericImage(systemImageName: "circle.fill")
                    .frame(width: 10, height: 10)
                    .foregroundStyle(NymColor.warning)
                    .padding(.horizontal, 8)
                rowSubtite(with: "gatewayInfo.datacenter".localizedString)
            case nil:
                GenericImage(systemImageName: "circle.fill")
                    .frame(width: 10, height: 10)
                    .foregroundStyle(NymColor.warning)
                    .padding(.horizontal, 8)
                rowSubtite(with: "noScore".localizedString)
            }
        }
    }
}

// MARK: - Performance section -
private extension GatewayDetailsView {
    func performanceSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            overAllPerformanceRow()
            separatorLine()
            serverLoadRow()
            separatorLine()
            uptimeRow()
            Spacer()
                .frame(height: 12)
            Text(formattedLastUpdate())
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Small.regular)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(NymColor.elevation)
        .cornerRadius(8)
    }

    func overAllPerformanceRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.overallPerformance".localizedString)
            Spacer()
            GenericImage(imageName: gateway.performance?.score.imageName)
                .frame(width: 16, height: 16)
                .padding(8)
            Text(gateway.performance?.score.localizedKey.localizedString ?? GatewayNodeScore.noScore.localizedKey.localizedString)
                .foregroundStyle(scoreOverallPerformanceImageColor(with: gateway.performance?.score ?? GatewayNodeScore.noScore))
                .textStyle(.Body.Medium.regular)
        }
    }

    func serverLoadRow() -> some View {
        HStack(spacing: 0) {
            rowTitle(with: "gatewayInfo.serverLoad".localizedString)
            Spacer()
            Text(gateway.performance?.load.localizedKey.localizedString ?? GatewayNodeScore.noScore.localizedKey.localizedString)
                .foregroundStyle(scoreLoadImageColor(with: gateway.performance?.load ?? GatewayNodeScore.noScore))
                .textStyle(.Body.Medium.regular)
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
private extension GatewayDetailsView {
    func serverInfoSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            buildVersionRow()
            separatorLine()
            identityRow()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(NymColor.elevation)
        .cornerRadius(8)
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
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Medium.regular)
                Spacer()
                    .frame(width: 16)
                GenericImage(imageName: "copy")
                    .frame(width: 24, height: 24)
                    .foregroundStyle(NymColor.primary)
                    .onTapGesture {
                        copyToPasteboard()
                    }
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
                        .background(NymColor.background)
                        .foregroundColor(NymColor.gray1)
                        .cornerRadius(8)
                        .transition(.opacity)
                        .padding(.trailing, 0)
                }
                .animation(.easeInOut, value: displayMessageOverlay)
            }
        }
    }
}

// MARK: - IP section -
private extension GatewayDetailsView {
    func ipSection() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            ipv4Rows()
            ipv6Rows()
            asnRow()
            separatorLine()
            asnNameRow()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .background(NymColor.elevation)
        .cornerRadius(8)
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
            rowSubtite(with: ip)
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
private extension GatewayDetailsView {
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
            connectionManager.entryGateway = .gateway(gateway)
        case .exit:
            connectionManager.exitRouter = .gateway(gateway)
        }
        path = .init()
    }
}

// MARK: - Helpers -
private extension GatewayDetailsView {
    // TODO: check if working
    func locationTitle() -> String {
        let parts = [
            gateway.location?.city,
            gateway.location?.region,
            countriesManager.country(with: gateway.location?.twoLetterIsoCountryCode)?.name
        ]
        .compactMap { $0 }
        .filter { !$0.isEmpty }
        return parts.joined(separator: ", ")
    }

    func scoreOverallPerformanceImageColor(with score: GatewayNodeScore) -> Color {
        switch score {
        case .noScore:
            NymColor.gray1
        case .low:
            NymColor.error
        case .medium:
            NymColor.warning
        case .high:
            NymColor.action
        case .offline:
            NymColor.gray1
        }
    }

    func scoreLoadImageColor(with score: GatewayNodeScore) -> Color {
        switch score {
        case .noScore:
            NymColor.gray1
        case .low:
            NymColor.action
        case .medium:
            NymColor.warning
        case .high:
            NymColor.error
        case .offline:
            NymColor.gray1
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
