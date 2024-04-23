import SwiftUI
import CountriesManager
import Theme
import UIComponents

public struct HopListView: View {
    @StateObject private var viewModel: HopListViewModel

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
    }
}

private extension HopListView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.type.selectHopLocalizedTitle,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateHome() }),
            isSmallScreen: viewModel.isSmallScreen
        )
    }

    @ViewBuilder
    func searchView() -> some View {
        SearchView(searchText: $viewModel.searchText)
            .padding(.horizontal, 24)
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
                isSelected: false
            )
        )
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(.all)
        .onTapGesture {
            viewModel.connectionSelect(with: country)
        }
    }
}
