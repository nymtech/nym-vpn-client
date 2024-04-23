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

                if let countryName = viewModel.countryName {
                    Text(countryName)
                        .foregroundStyle(NymColor.sysOnSurface)
                        .textStyle(.Body.Large.primary)
                }

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
    @ViewBuilder func flagOrBoltImage() -> some View {
        if viewModel.isQuickest {
            BoltImage()
        } else if let countryCode = viewModel.countryCode {
            FlagImage(countryCode: countryCode)
        }
    }
}
