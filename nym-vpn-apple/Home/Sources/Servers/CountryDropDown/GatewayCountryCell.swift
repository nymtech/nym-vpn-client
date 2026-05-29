import SwiftUI
import ConnectionManager
import ConnectionTypes
import GatewayManager
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayCountryCell: View {
    private let country: NymCountry
    private let servers: [GatewayNode]
    private let hopType: HopType
    private let isSearching: Bool
    private let cornerRadius: CGFloat = 16

    @EnvironmentObject private var gatewayManager: GatewayManager
    @State private var isButtonHovered = false
    @State private var isExpanded: Bool
    @State private var isCountrySelected = false
    @Binding private var path: NavigationPath
    @Binding private var scrollToModel: GatewayScrollToModel
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    private var infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?

    public init(
        country: NymCountry,
        servers: [GatewayNode],
        type: HopType,
        path: Binding<NavigationPath>,
        scrollToModel: Binding<GatewayScrollToModel>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?,
        isSearching: Bool = false
    ) {
        self.country = country
        self.servers = servers
        self.hopType = type
        self.isSearching = isSearching
        self.infoButtonTapCompletion = infoButtonTapCompletion
        _path = path
        _scrollToModel = scrollToModel
        _entryGateway = entryGateway
        _exitRouter = exitRouter

        let unwrappedScrollToModel = scrollToModel.wrappedValue
        let selectedServer = servers.first { $0.id == unwrappedScrollToModel.serverId }
        let shouldExpand = unwrappedScrollToModel.shouldExpand(
            countryCode: country.code,
            region: nil,
            server: selectedServer
        )
        _isExpanded = State(initialValue: shouldExpand)
        let shouldSelect = unwrappedScrollToModel.countryCode == country.code && unwrappedScrollToModel.isCountry
        _isCountrySelected = State(initialValue: shouldSelect)
    }

    public var body: some View {
        VStack(spacing: 0) {
            countryRow()
                .overlay {
                    UnevenRoundedRectangle(
                        topLeadingRadius: cornerRadius,
                        bottomLeadingRadius: isExpanded ? 0 : cornerRadius,
                        bottomTrailingRadius: isExpanded ? 0 : cornerRadius,
                        topTrailingRadius: cornerRadius
                    )
                    .inset(by: 0.5)
                    .stroke(isCountrySelected ? Color.Nym.primary : .clear, lineWidth: 1)
                    .allowsHitTesting(false)
                }
                .animation(.default, value: isCountrySelected)
                .id(GatewayScrollToModel.country(code: country.code).scrollToIdentifier)
            if isExpanded {
                expandedContent()
            }
        }
        .background {
            RoundedRectangle(cornerRadius: cornerRadius)
                .fill(Color.Nym.surface)
        }
        .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
        .padding(.horizontal, NymSpacing.large)
        .padding(.bottom, NymSpacing.small)
    }
}

private extension GatewayCountryCell {
    @ViewBuilder
    func expandedContent() -> some View {
        Divider()
            .frame(height: 1)
            .overlay(Color.Nym.divider)

        if !country.regions.isEmpty && gatewayManager.shouldDisplayRegion(with: country.code) {
            ForEach(Array(country.regions.enumerated()), id: \.element.self) { index, region in
                if index > 0 {
                    Divider()
                        .frame(height: 1)
                        .overlay(Color.Nym.divider)
                }
                GatewayRegionCell(
                    hopType: hopType,
                    country: country,
                    region: region.name,
                    servers: servers.filter { $0.location?.region == region.name },
                    infoButtonTapCompletion: infoButtonTapCompletion,
                    path: $path,
                    entryGateway: $entryGateway,
                    exitRouter: $exitRouter,
                    scrollToModel: $scrollToModel,
                    bottomCornerRadius: index == country.regions.count - 1 ? cornerRadius : 0
                )
                .id(
                    GatewayScrollToModel.region(
                        countryCode: country.code,
                        region: region.name
                    )
                    .scrollToIdentifier
                )
            }
        } else {
            ForEach(Array(servers.enumerated()), id: \.element.id) { index, server in
                if index > 0 {
                    Divider()
                        .frame(height: 1)
                        .overlay(Color.Nym.divider)
                }
                GatewayCell(
                    server: server,
                    type: hopType,
                    path: $path,
                    scrollToModel: $scrollToModel,
                    bottomCornerRadius: index == servers.count - 1 ? cornerRadius : 0,
                    infoButtonTapCompletion: { server in
                        infoButtonTapCompletion?(server)
                    }
                )
                .id(GatewayScrollToModel.server(id: server.id).scrollToIdentifier)
            }
        }
    }

    @ViewBuilder
    func countryRow() -> some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                FlagImage(countryCode: country.code)
                    .padding(.leading, NymSpacing.large)
                Text(country.name)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyLarge)
                    .padding(.leading, NymSpacing.medium)
                Spacer()
            }
            .frame(maxHeight: .infinity)
            .contentShape(Rectangle())
            .accessibilityElement(children: .combine)
            .accessibilityLabel("\(country.name) \(servers.count) \("servers".localizedString)")
            .accessibilityValue(isCountrySelected ? "selected".localizedString : "")
            .accessibilityAddTraits([.isButton])
            .onTapGesture {
                countrySelectTapAction()
            }
            .accessibilityAction {
                countrySelectTapAction()
            }

            chevron()
                .padding(.trailing, NymSpacing.large)
                .frame(maxHeight: .infinity)
                .contentShape(Rectangle())
                .accessibilityElement(children: .combine)
                .accessibilityLabel("gatewaySelector.expandServers".localizedString)
                .accessibilityAddTraits([.isButton])
                .onTapGesture {
                    expandDidTap()
                }
                .accessibilityAction {
                    expandDidTap()
                }
        }
        .frame(height: 64)
        .background(isButtonHovered ? Color.Nym.background.opacity(0.3) : Color.clear)
        .onHover { newValue in
            isButtonHovered = newValue
        }
    }

    func chevron() -> some View {
        Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
            .font(.system(size: 14, weight: .semibold))
            .foregroundStyle(isExpanded ? Color.Nym.primary : Color.Nym.textSecondary)
            .frame(width: 24, height: 24)
            .animation(.easeInOut(duration: 0.2), value: isExpanded)
    }
}

private extension GatewayCountryCell {
    func countrySelectTapAction() {
        ImpactGenerator.shared.softImpact()
        switch hopType {
        case .entry:
            entryGateway = .country(country.code)
        case .exit:
            exitRouter = .country(country.code)
        }
        path = .init()
    }

    func expandDidTap() {
        ImpactGenerator.shared.softImpact()
        withAnimation(.easeInOut(duration: 0.2)) {
            isExpanded.toggle()
        }
    }
}
