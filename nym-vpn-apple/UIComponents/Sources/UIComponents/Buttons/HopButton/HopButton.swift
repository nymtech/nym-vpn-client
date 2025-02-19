import SwiftUI
import CountriesManager
import Theme

public struct HopButton: View {
    @ObservedObject var viewModel: HopButtonViewModel

    public init(viewModel: HopButtonViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        StrokeBorderView(strokeTitle: viewModel.hopType.hopLocalizedTitle, strokeTitleLeftMargin: 30) {
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
        }
    }

    func titleText(with text: String) -> some View {
        Text(text)
            .lineLimit(1)
            .foregroundStyle(NymColor.sysOnSurface)
            .textStyle(.BodyLegacy.Large.semibold)
    }
}
