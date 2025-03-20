import SwiftUI
import Constants
import CountriesManagerTypes
import Device
import ExternalLinkManager
import Theme
import UIComponents

public struct GatewaysView: View {
    @StateObject private var viewModel: GatewaysViewModel
    @FocusState private var isSearchFocused: Bool

    public init(viewModel: GatewaysViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)

            searchView()
                .frame(maxWidth: Device.type == .ipad ? 358 : 390)
            Spacer()
                .frame(height: 24)

            ScrollViewReader { proxy in
                ScrollView {
                    countriesGatewaysList()
                    noSearchResultsView()
                    foundCountriesList()
                    foundGatewaysList()
                }
                .scrollIndicators(.hidden)
                .frame(maxWidth: Device.type == .ipad ? 358 : 390)
                .ignoresSafeArea(.all)
                .onChange(of: viewModel.scrollToServer) { _ in
                    guard let server = viewModel.scrollToServer else { return }
                    withAnimation {
                        proxy.scrollTo(server.id, anchor: .center)
                    }
                }
            }
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .overlay {
            if viewModel.isGeolocationModalDisplayed {
                LocationInfoView(
                    viewModel: LocationInfoViewModel(
                        externalLinkManager: ExternalLinkManager.shared,
                        isDisplayed: $viewModel.isGeolocationModalDisplayed
                    )
                )
                .transition(.opacity)
                .animation(.easeInOut, value: viewModel.isGeolocationModalDisplayed)
            }
        }
        .overlay {
            if viewModel.isServerInfoModalDisplayed, let server = viewModel.serverInfoModalServer {
                GatewayInfoModal(server: server, isDisplayed: $viewModel.isServerInfoModalDisplayed)
                    .transition(.opacity)
                    .animation(.easeInOut, value: viewModel.isServerInfoModalDisplayed)
            }
        }
        .onTapGesture {
            isSearchFocused = false
        }
    }
}

private extension GatewaysView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.type.selectHopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() }),
            rightButton: CustomNavBarButton(type: .info, action: { viewModel.displayInfoTooltip() })
        )
    }

    @ViewBuilder
    func searchView() -> some View {
        SearchView(searchText: $viewModel.searchText, isSearchFocused: $isSearchFocused)
            .padding(.horizontal, 16)
    }

    @ViewBuilder
    func countriesGatewaysList() -> some View {
        if viewModel.searchText.count < viewModel.minimumSearchSymbols {
            ForEach(viewModel.countries, id: \.name) { country in
                GatewayCountryDropDown(
                    country: country,
                    servers: viewModel.gatewaysInCountry(with: country.code),
                    type: viewModel.type,
                    path: $viewModel.path,
                    isServerModalDisplayed: $viewModel.isServerInfoModalDisplayed,
                    serverInfoModalServer: $viewModel.serverInfoModalServer,
                    scrollToServer: $viewModel.scrollToServer
                )
            }
        }
    }

    @ViewBuilder
    func noSearchResultsView() -> some View {
        if viewModel.searchText.count >= viewModel.minimumSearchSymbols,
           viewModel.foundGateways.isEmpty,
           viewModel.foundCountries.isEmpty {
            VStack {
                Text("search.noResults".localizedString)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Large.regular)
                Spacer()
                    .frame(height: 16)
                Text("search.noResultsSubtitle".localizedString)
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Large.regular)
                Spacer()
                    .frame(height: 4)

                contactUsForHelpLinkView()
                Spacer()
                    .frame(height: 4)
            }
        }
    }

    @ViewBuilder
    func contactUsForHelpLinkView() -> some View {
        if let newSupportRequestURL = URL(string: Constants.newSupportRequest.rawValue),
           let operatorURL = URL(string: Constants.operatorDocs.rawValue) {
            HStack(spacing: 0) {
                Link(destination: newSupportRequestURL) {
                    Text("search.contactUsForHelp".localizedString)
                        .underline()
                        .foregroundStyle(NymColor.gray1)
                        .textStyle(.Body.Large.regular)
                }

                Text(" \("search.or".localizedString) ")
                    .foregroundStyle(NymColor.gray1)
                    .textStyle(.Body.Large.regular)

                Link(destination: operatorURL) {
                    Text("search.howToRunGateway".localizedString)
                        .underline()
                        .foregroundStyle(NymColor.gray1)
                        .textStyle(.Body.Large.regular)
                }
            }
        }
    }

    @ViewBuilder
    func foundCountriesList() -> some View {
        ForEach(viewModel.foundCountries, id: \.name) { country in
            GatewayCountryDropDown(
                country: country,
                servers: viewModel.gatewaysInCountry(with: country.code),
                type: viewModel.type,
                path: $viewModel.path,
                isServerModalDisplayed: $viewModel.isServerInfoModalDisplayed,
                serverInfoModalServer: $viewModel.serverInfoModalServer,
                scrollToServer: $viewModel.scrollToServer,
                isSearching: true
            )
        }
        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func foundGatewaysList() -> some View {
        ForEach(viewModel.foundGateways, id: \.id) { server in
            GatewayCell(
                server: server,
                type: viewModel.type,
                path: $viewModel.path,
                isServerModalDisplayed: $viewModel.isServerInfoModalDisplayed,
                serverInfoModalServer: $viewModel.serverInfoModalServer,
                isSearching: true
            )
        }
    }
}
