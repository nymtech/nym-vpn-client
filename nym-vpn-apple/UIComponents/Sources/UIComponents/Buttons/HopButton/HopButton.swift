import SwiftUI
import CountriesManager
import Theme

public struct HopButton: View {
    @ObservedObject var viewModel: HopButtonViewModel

    public init(viewModel: HopButtonViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        StrokeBorderView(strokeTitle: viewModel.hopType.hopLocalizedTitle) {
            HStack {
                flagOrBoltImage()
                    .padding(.horizontal, 12)

                titleText(with: viewModel.name)

                Spacer()
                Image(viewModel.arrowImageName, bundle: .module)
                    .resizable()
                    .frame(width: 24, height: 24)
                    .padding(16)
            }
        }
    }
}

private extension HopButton {
    @ViewBuilder
    func flagOrBoltImage() -> some View {
        if viewModel.isQuickest {
            BoltImage()
        } else if let countryCode = viewModel.countryCode {
            FlagImage(countryCode: countryCode)
        } else if viewModel.isGateway {
            Image(systemName: "network")
                .resizable()
                .frame(width: 24, height: 24)
                .cornerRadius(50)
                .foregroundStyle(NymColor.sysOnSurface)
        }
    }

    func titleText(with text: String) -> some View {
        Text(text)
            .foregroundStyle(NymColor.sysOnSurface)
            .textStyle(.Body.Large.semibold)
    }
}
