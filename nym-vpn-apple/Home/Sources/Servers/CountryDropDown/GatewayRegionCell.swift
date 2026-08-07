import SwiftUI
import ConnectionManager
import ConnectionTypes
import GatewayManager
import ImpactGenerator
import Theme
import UIComponents

public struct GatewayRegionCell: View {
    private let hopType: HopType
    private let country: NymCountry
    private let region: String
    private let servers: [GatewayNode]
    private let bottomCornerRadius: CGFloat
    @EnvironmentObject private var gatewayManager: GatewayManager
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    @Binding private var path: NavigationPath
    @Binding private var scrollToModel: GatewayScrollToModel
    @State private var isExpanded: Bool
    @State private var isButtonHovered = false
    @State private var isRegionSelected = false
    private var infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?

    public var body: some View {
        VStack(spacing: 0) {
            regionRow()
            if isExpanded {
                ForEach(Array(servers.enumerated()), id: \.element.id) { index, server in
                    Divider()
                        .frame(height: 1)
                        .overlay(Color.Nym.divider)
                    GatewayCell(
                        server: server,
                        type: hopType,
                        path: $path,
                        scrollToModel: $scrollToModel,
                        bottomCornerRadius: index == servers.count - 1 ? bottomCornerRadius : 0,
                        infoButtonTapCompletion: { server in
                            infoButtonTapCompletion?(server)
                        }
                    )
                    .id(GatewayScrollToModel.server(id: server.id).scrollToIdentifier)
                }
            }
        }
    }

    public init(
        hopType: HopType,
        country: NymCountry,
        region: String,
        servers: [GatewayNode],
        infoButtonTapCompletion: (@Sendable @MainActor (GatewayNode) -> Void)?,
        path: Binding<NavigationPath>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        scrollToModel: Binding<GatewayScrollToModel>,
        bottomCornerRadius: CGFloat = 0,
        isInitiallyExpanded: Bool = false
    ) {
        self.hopType = hopType
        self.country = country
        self.region = region
        self.servers = servers
        self.infoButtonTapCompletion = infoButtonTapCompletion
        self.bottomCornerRadius = bottomCornerRadius
        _path = path
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        _scrollToModel = scrollToModel

        let unwrappedScrollToModel = scrollToModel.wrappedValue
        let selectedServer = servers.first { $0.id == unwrappedScrollToModel.serverId }
        let shouldExpand = unwrappedScrollToModel.shouldExpand(
            countryCode: country.code,
            region: region,
            server: selectedServer
        )
        _isExpanded = State(initialValue: shouldExpand || isInitiallyExpanded)
        let shouldSelect = unwrappedScrollToModel.region == region && unwrappedScrollToModel.isRegion
        _isRegionSelected = State(initialValue: shouldSelect)
    }
}

private extension GatewayRegionCell {
    @ViewBuilder
    func regionRow() -> some View {
        HStack(spacing: 0) {
            HStack(spacing: 0) {
                Text(region)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyLarge)
                    .padding(.leading, NymSpacing.large)
                Spacer()
            }
            .frame(maxHeight: .infinity)
            .contentShape(Rectangle())
            .accessibilityElement(children: .combine)
            .accessibilityLabel("\(region) \(servers.count) \("servers".localizedString)")
            .accessibilityValue(isRegionSelected ? "selected".localizedString : "")
            .accessibilityAddTraits([.isButton])
            .onTapGesture {
                regionSelectTapAction()
            }
            .accessibilityAction {
                regionSelectTapAction()
            }

            chevron()
                .padding(.trailing, NymSpacing.large)
                .frame(maxHeight: .infinity)
                .contentShape(Rectangle())
                .accessibilityElement(children: .combine)
                .accessibilityLabel("gatewaySelector.expandServers".localizedString)
                .accessibilityAddTraits([.isButton])
                .onTapGesture {
                    expandTapAction()
                }
                .accessibilityAction {
                    expandTapAction()
                }
        }
        .frame(height: 56)
        .padding(.leading, NymSpacing.medium)
        .background(isButtonHovered ? Color.Nym.background.opacity(0.3) : Color.clear)
        .overlay {
            let radius = isExpanded ? 0 : bottomCornerRadius
            UnevenRoundedRectangle(
                topLeadingRadius: 0,
                bottomLeadingRadius: radius,
                bottomTrailingRadius: radius,
                topTrailingRadius: 0
            )
            .inset(by: 0.5)
            .stroke(isRegionSelected ? Color.Nym.primary : .clear, lineWidth: 1)
            .allowsHitTesting(false)
        }
        .animation(.default, value: isRegionSelected)
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

public extension GatewayRegionCell {
    func expandTapAction() {
        ImpactGenerator.shared.softImpact()
        withAnimation(.easeInOut(duration: 0.2)) {
            isExpanded.toggle()
        }
    }

    func regionSelectTapAction() {
        ImpactGenerator.shared.softImpact()
        switch hopType {
        case .entry:
            entryGateway = .region(countryCode: country.code, region: region)
        case .exit:
            exitRouter = .region(countryCode: country.code, region: region)
        }
        path = .init()
    }
}
