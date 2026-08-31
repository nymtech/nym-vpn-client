import SwiftUI
import Constants
import ConnectionTypes
import ExternalLinkManager
import GatewayManager
import Routes
import Settings
import Theme
import UIComponents

public struct GatewaysView: View {
    @StateObject private var viewModel: GatewaysViewModel
    // Favorites live in GatewayManager; observing it here keeps the filtered lists in sync.
    @EnvironmentObject private var gatewayManager: GatewayManager
    @FocusState private var isSearchFocused: Bool

    private var favoritesState: ServersFavoritesState {
        viewModel.favoritesState
    }

    private var displayedCountries: [NymCountry] {
        switch favoritesState.filter {
        case .favorites:
            viewModel.countries.filter {
                favoritesState.isFavorite(.country($0.code)) || !servers(in: $0).isEmpty
            }
        case .recent:
            // Recents are a flat, core-ordered node list — see recentGatewaysList().
            []
        case .allServers:
            viewModel.countries
        }
    }

    /// Nodes listed under a country cell. The favorites tab lists only starred nodes — a
    /// starred country is itself the favorite and renders as a childless row, so the tab
    /// never shows an unstarred entry.
    private func servers(in country: NymCountry) -> [GatewayNode] {
        let servers = viewModel.gatewaysInCountry(with: country.code)
        guard favoritesState.filter == .favorites else { return servers }
        return servers.filter { favoritesState.isFavorite(.gateway($0.id)) }
    }

    private var entryGatewayBinding: Binding<EntryGateway> {
        Binding(
            get: { viewModel.connectionManager.entryGateway },
            set: { viewModel.applyEntrySelection($0) }
        )
    }

    private var exitRouterBinding: Binding<ExitRouter> {
        Binding(
            get: { viewModel.connectionManager.exitRouter },
            set: { viewModel.connectionManager.applyExplicitExit($0) }
        )
    }

    /// Autoclosure keeps the model alive across parent re-renders: the expression only
    /// runs when this navigation destination's `@StateObject` is first created, so the
    /// model's init side effects (gateway load, scroll, favorites fetch) fire once.
    public init(viewModel: @autoclosure @escaping () -> GatewaysViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel())
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            optionalQuicMessage()
            searchAndFilterHeader()

            ScrollViewReader { proxy in
                ScrollView {
                    safestRow()
                    randomRow()
                    recentGatewaysList()
                    favoriteGatewaysList()
                    countriesGatewaysList()
                    noSearchResultsView()
                    foundCountriesList()
                    foundRegionsList()
                    foundGatewaysList()
                }
                .scrollDismissesKeyboard(.immediately)
                .scrollIndicators(.never)
                .frame(maxWidth: MagicNumbers.maxWidth)
                .ignoresSafeArea(.all)
                .onReceive(viewModel.$shouldScroll.filter { $0 }) { _ in
                    Task { @MainActor in
                        await Task.yield() // let SwiftUI lay out with the new data
                        withAnimation {
                            proxy.scrollTo(viewModel.scrollToModel.scrollToIdentifier, anchor: .top)
                        }
                        viewModel.shouldScroll = false
                    }
                }
            }
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .environmentObject(favoritesState)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
        .overlay {
            if viewModel.isGeolocationModalDisplayed {
                LocationInfoView(type: viewModel.type, isDisplayed: $viewModel.isGeolocationModalDisplayed)
                    .transition(.opacity)
                    .animation(.easeInOut, value: viewModel.isGeolocationModalDisplayed)
            }
        }
        .onTapGesture {
            isSearchFocused = false
        }
        .onChange(of: favoritesState.filter) { _, newFilter in
            switch newFilter {
            case .recent:
                Task { await viewModel.updateRecents() }
            case .favorites:
                Task { await viewModel.gatewayManager.updateFavorites() }
            case .allServers:
                break
            }
            // Results are scoped to the tab, so an active search has to be re-run against the new one.
            viewModel.searchCountriesGateways()
        }
        .task {
            try? await Task.sleep(nanoseconds: 350_000_000)
            guard !Task.isCancelled else { return }
            isSearchFocused = true
        }
    }
}

private extension GatewaysView {
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.type.hopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() }),
            rightButton: CustomNavBarButton(type: .info, action: { viewModel.displayInfoTooltip() })
        )
    }

    func navigateBack() {
        if isSearchFocused {
            isSearchFocused = false
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
                viewModel.navigateHome()
            }
        } else {
            viewModel.navigateHome()
        }
    }

    @ViewBuilder
    func optionalQuicMessage() -> some View {
        if viewModel.shouldShowQuic {
            Spacer()
                .frame(height: 24)
            VStack(alignment: .leading, spacing: 0) {
                HStack(spacing: 0) {
                    Text(quicText())
                        .foregroundStyle(Color.Nym.textSecondary)
                        .nymTextStyle(.bodySmall)
                }
            }
            .padding(.horizontal, 16)
            .frame(maxWidth: MagicNumbers.maxWidth)
            .environment(\.openURL, OpenURLAction { url in
                if url == URL(string: "app://enable-quic") {
                    viewModel.path.append(HomeLink.settings)
                    viewModel.path.append(SettingLink.censorship)
                    return .handled
                }
                return .systemAction
            })
        }
    }

    func quicText() -> AttributedString {
        let first = AttributedString("gatewaysView.quic1".localizedString)
        var secondAttr = AttributedString("gatewaysView.quic2".localizedString)
        secondAttr.underlineStyle = .single
        secondAttr.foregroundColor = Color.Nym.textPrimary
        secondAttr.link = URL(string: "app://enable-quic")
        return first + AttributedString(" ") + secondAttr
    }

    func searchView() -> some View {
        SearchView(searchText: $viewModel.searchText, isSearchFocused: $isSearchFocused)
            .padding(.horizontal, 16)
    }

    func serverFilterSelector() -> some View {
        ServerFilterSelector(
            selection: Binding(
                get: { favoritesState.filter },
                set: { favoritesState.filter = $0 }
            )
        )
        .padding(.horizontal, 16)
    }

    func searchAndFilterHeader() -> some View {
        VStack(spacing: 0) {
            Spacer()
                .frame(height: 24)
            searchView()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
                .frame(height: 16)
            countSummaryHeader()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
                .frame(height: 12)
            serverFilterSelector()
                .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
                .frame(height: 12)
        }
    }

    @ViewBuilder
    func countSummaryHeader() -> some View {
        let countriesCount = viewModel.countries.count
        let nodesCount = viewModel.gateways.count
        if countriesCount > 0 {
            HStack(spacing: 0) {
                Text("\(countriesCount) \("gatewaysView.countries".localizedString) · \(nodesCount) \("gatewaysView.nodes".localizedString)")
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodySmall)
                Spacer()
            }
            .padding(.horizontal, 16)
        }
    }

    @ViewBuilder
    func safestRow() -> some View {
        if viewModel.searchText.count < viewModel.minimumSearchSymbols, favoritesState.filter == .allServers {
            switch viewModel.type {
            case .entry:
                GatewaySafestCell(
                    type: .entry,
                    path: $viewModel.path,
                    entryGateway: entryGatewayBinding,
                    exitRouter: exitRouterBinding
                )
            case .exit:
                GatewaySafestCell(
                    type: .exit,
                    path: $viewModel.path,
                    entryGateway: entryGatewayBinding,
                    exitRouter: exitRouterBinding,
                    onTap: { viewModel.applyExitAutoTap() }
                )
            }
        }
    }

    @ViewBuilder
    func randomRow() -> some View {
        if viewModel.searchText.count < viewModel.minimumSearchSymbols, favoritesState.filter == .allServers {
            switch viewModel.type {
            case .entry:
                GatewayRandomCell(
                    type: .entry,
                    path: $viewModel.path,
                    entryGateway: entryGatewayBinding,
                    exitRouter: exitRouterBinding
                )
            case .exit:
                GatewayRandomCell(
                    type: .exit,
                    path: $viewModel.path,
                    entryGateway: entryGatewayBinding,
                    exitRouter: exitRouterBinding,
                    onTap: { viewModel.applyExitRandomTap() }
                )
            }
        }
    }

    @ViewBuilder
    func countriesGatewaysList() -> some View {
        if viewModel.searchText.count < viewModel.minimumSearchSymbols {
            ForEach(displayedCountries, id: \.name) { country in
                let servers = servers(in: country)
                // A starred country with no starred nodes is still a favorite — keep its row.
                if !servers.isEmpty || favoritesState.filter == .favorites {
                    GatewayCountryCell(
                        country: country,
                        servers: servers,
                        type: viewModel.type,
                        path: $viewModel.path,
                        scrollToModel: $viewModel.scrollToModel,
                        entryGateway: entryGatewayBinding,
                        exitRouter: exitRouterBinding,
                        infoButtonTapCompletion: { gateway in
                            viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                        },
                        // Favorites tab opens expanded, else the starred nodes stay hidden behind a chevron.
                        isInitiallyExpanded: favoritesState.filter == .favorites
                    )
                    // Cell expansion is @State seeded at init; switching tabs must re-seed it.
                    .id(favoritesState.filter)
                }
            }
        }
    }

    /// Nodes core recorded as recently connected, newest first.
    @ViewBuilder
    func recentGatewaysList() -> some View {
        if favoritesState.filter == .recent, viewModel.searchText.count < viewModel.minimumSearchSymbols {
            if viewModel.recentGateways.isEmpty {
                Text("gatewaysView.filter.noRecents".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodyLarge)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding(.horizontal, 16)
            } else {
                ForEach(viewModel.recentGateways, id: \.id) { server in
                    GatewayCell(
                        server: server,
                        type: viewModel.type,
                        path: $viewModel.path,
                        scrollToModel: .constant(.empty),
                        isSearching: true,
                        infoButtonTapCompletion: { gateway in
                            viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                        }
                    )
                }
            }
        }
    }

    /// Starred nodes and countries render through `countriesGatewaysList()`; only the
    /// empty state is left here.
    @ViewBuilder
    func favoriteGatewaysList() -> some View {
        if favoritesState.filter == .favorites,
           viewModel.searchText.count < viewModel.minimumSearchSymbols,
           displayedCountries.isEmpty {
            Text("gatewaysView.filter.noFavorites".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyLarge)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 16)
        }
    }

    @ViewBuilder
    func noSearchResultsView() -> some View {
        if viewModel.searchText.count >= viewModel.minimumSearchSymbols,
           viewModel.foundGateways.isEmpty,
           viewModel.foundCountries.isEmpty {
            VStack(alignment: .leading, spacing: 0) {
                Text("search.noResults".localizedString)
                    .foregroundStyle(Color.Nym.textPrimary)
                    .nymTextStyle(.bodyLarge)
                Spacer()
                    .frame(height: 16)
                Text("search.noResultsSubtitle".localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodyLarge)
                Spacer()
                    .frame(height: 8)

                contactUsForHelpLinkView()
                Spacer()
                    .frame(height: 4)
            }
            .padding(.horizontal, 16)
        }
    }

    @ViewBuilder
    func contactUsForHelpLinkView() -> some View {
        if let attributtedText = contactUsForHelpAttributedString() {
            Text(attributtedText)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyLarge)
        }
    }

    func contactUsForHelpAttributedString() -> AttributedString? {
        guard let newSupportRequestURL = URL(string: Constants.newSupportRequest.rawValue),
              let operatorURL = URL(string: Constants.operatorDocs.rawValue)
        else {
            return nil
        }

        let contactUs = "search.contactUsForHelp".localizedString
        let orText = "search.or".localizedString
        let howToRun = "search.howToRunGateway".localizedString
        let markdown = "[\(contactUs)](\(newSupportRequestURL.absoluteString)) \(orText) [\(howToRun)](\(operatorURL.absoluteString))."

        let options = AttributedString.MarkdownParsingOptions(
            interpretedSyntax: .inlineOnlyPreservingWhitespace
        )

        guard var attributed = try? AttributedString(markdown: markdown, options: options)
        else {
            return nil
        }

        for run in attributed.runs where run.link != nil {
            attributed[run.range].underlineStyle = .single
            attributed[run.range].foregroundColor = Color.Nym.textSecondary
        }
        return attributed
    }

    @ViewBuilder
    func foundCountriesList() -> some View {
        ForEach(viewModel.foundCountries, id: \.name) { country in
            // Same tab scoping as the unsearched list — a matched country on the favorites tab
            // must not unfold into its unstarred nodes.
            let servers = servers(in: country)
            if !servers.isEmpty {
                GatewayCountryCell(
                    country: country,
                    servers: servers,
                    type: viewModel.type,
                    path: $viewModel.path,
                    scrollToModel: $viewModel.scrollToModel,
                    entryGateway: entryGatewayBinding,
                    exitRouter: exitRouterBinding,
                    infoButtonTapCompletion: { gateway in
                        viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                    },
                    isSearching: true
                )
            }
        }
        Spacer()
            .frame(height: 24)
    }

    func foundGatewaysList() -> some View {
        ForEach(viewModel.foundGateways, id: \.id) { server in
            GatewayCell(
                server: server,
                type: viewModel.type,
                path: $viewModel.path,
                scrollToModel: .constant(.empty),
                isSearching: true,
                infoButtonTapCompletion: { gateway in
                    viewModel.path.append(HomeLink.gatewayDetails(gateway: gateway, hopType: viewModel.type))
                }
            )
        }
    }

    @ViewBuilder
    func foundRegionsList() -> some View {
        ForEach(viewModel.foundRegions, id: \.region) { (country: NymCountry, region: String) in
            let servers = viewModel.gateways.filter { $0.location?.region == region }
            if !servers.isEmpty {
                // Region cells are built to sit inside a country card, so a standalone search
                // result has to bring its own.
                GatewayRegionCell(
                    hopType: viewModel.type,
                    country: country,
                    region: region,
                    servers: servers,
                    infoButtonTapCompletion: { _ in },
                    path: $viewModel.path,
                    entryGateway: entryGatewayBinding,
                    exitRouter: exitRouterBinding,
                    scrollToModel: .constant(.empty),
                    bottomCornerRadius: 16
                )
                .background {
                    RoundedRectangle(cornerRadius: 16)
                        .fill(Color.Nym.surface)
                }
                .clipShape(RoundedRectangle(cornerRadius: 16))
                .padding(.horizontal, 16)
                .padding(.bottom, 8)
            }
        }
    }
}
