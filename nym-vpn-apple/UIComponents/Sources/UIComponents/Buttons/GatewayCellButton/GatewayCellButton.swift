import SwiftUI
import Theme

public struct GatewayCellButton: View {
    private let viewModel: GatewayCellButtonViewModel

    public init(viewModel: GatewayCellButtonViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        HStack {
            gatewayImage()
                .padding(.trailing, 12)

            Text(viewModel.title)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.BodyLegacy.Large.regular)

            Spacer()
            selectedTitleView()
        }
        .contentShape(
            RoundedRectangle(cornerRadius: 8)
        )
        .frame(height: 56, alignment: .center)
        .cornerRadius(8)
        .padding(.horizontal, 24)
    }
}

private extension GatewayCellButton {

    @ViewBuilder
    func gatewayImage() -> some View {
        switch viewModel.type {
        case .fastest:
            BoltImage()
        case .country:
            if let country = viewModel.type.country {
                FlagImage(countryCode: country.code)
            }
        case .gateway:
            networkImage()
        }
    }

    @ViewBuilder
    func selectedTitleView() -> some View {
        if viewModel.isSelected {
            Text(viewModel.selectedTitle)
                .textStyle(.LabelLegacy.Small.primary)
                .padding(.trailing, 24)
        }
    }

    func networkImage() -> some View {
        GenericImage(systemImageName: "network")
            .frame(width: 24, height: 24)
            .cornerRadius(50)
            .foregroundStyle(NymColor.sysOnSurface)
    }
}
