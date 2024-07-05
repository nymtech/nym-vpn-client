import SwiftUI
import CountriesManager
import Theme
import UIComponents

public struct HopListView: View {
    @StateObject private var viewModel: HopListViewModel
    @FocusState private var isSearchFocused: Bool

    public init(viewModel: HopListViewModel) {
        _viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)

            searchView()
            Spacer()
                .frame(height: 24)

            ScrollView {
                noSearchResultsView()
                quickestConnection()
                availableCountryList()
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .ignoresSafeArea(.all)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
        .onTapGesture {
            isSearchFocused = false
        }
    }
}

private extension HopListView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.type.selectHopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() })
        )
    }

    @ViewBuilder
    func searchView() -> some View {
        SearchView(searchText: $viewModel.searchText, isSearchFocused: $isSearchFocused)
            .padding(.horizontal, 24)
    }

    @ViewBuilder
    func noSearchResultsView() -> some View {
        if !viewModel.searchText.isEmpty && viewModel.countries?.isEmpty ?? true {
            VStack {
                Text(viewModel.noResultsText)
                    .textStyle(.Body.Medium.regular)
                    .padding(.top, 96)
                Spacer()
            }
        }
    }

    @ViewBuilder
    func quickestConnection() -> some View {
        if let country = viewModel.quickestCountry {
            CountryCellButton(
                viewModel:
                    CountryCellButtonViewModel(
                        type: .fastest(country: country),
                        isSelected: false
                    )
            )
            .onTapGesture {
                viewModel.quickestConnectionSelect(with: country)
            }
        }
    }

    @ViewBuilder
    func availableCountryList() -> some View {
        if let countries = viewModel.countries {
            ForEach(countries, id: \.name) { country in
                countryButton(with: country)
            }
        }
    }

    @ViewBuilder
    func countryButton(with country: Country) -> some View {
        CountryCellButton(
            viewModel: CountryCellButtonViewModel(
                type: .country(
                    country: country
                ),
                isSelected: viewModel.isCountrySelected(countryCode: country.code)
            )
        )
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(.all)
        .onTapGesture {
            viewModel.connectionSelect(with: country)
        }
    }
}
