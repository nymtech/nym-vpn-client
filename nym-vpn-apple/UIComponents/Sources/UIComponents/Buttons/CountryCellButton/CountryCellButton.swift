import SwiftUI
import Theme

public struct CountryCellButton: View {
    private let viewModel: CountryCellButtonViewModel

    public init(viewModel: CountryCellButtonViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        HStack {
            flagOrBoltImage()

            Text(viewModel.title)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.Body.Large.primary)

            Spacer()
            selectedTitleView()
        }
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
        )
        .frame(height: 56, alignment: .center)
        .background(viewModel.backgroundColor)
        .cornerRadius(8)
        .padding(.horizontal, 24)
    }
}

private extension CountryCellButton {

    @ViewBuilder
    func flagOrBoltImage() -> some View {
        switch viewModel.type {
        case .fastest:
            BoltImage()
        case .country:
            FlagImage(countryCode: viewModel.type.country.code)
        }
    }

    @ViewBuilder
    func selectedTitleView() -> some View {
        if viewModel.isSelected {
            Text(viewModel.selectedTitle)
                .textStyle(.Label.Small.primary)
                .padding(.trailing, 24)
        }
    }
}
