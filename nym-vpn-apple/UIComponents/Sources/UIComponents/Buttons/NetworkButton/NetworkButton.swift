import SwiftUI
import Theme

public struct NetworkButton: View {
    @StateObject private var viewModel: NetworkButtonViewModel

    public init(viewModel: NetworkButtonViewModel) {
        self._viewModel = StateObject(wrappedValue: viewModel)
    }

    public var body: some View {
        VStack {
            HStack {
                Image(viewModel.selectionImageName, bundle: .module)
                    .foregroundStyle(viewModel.selectionImageColor)
                    .padding(.leading, 16)

                Image(viewModel.imageName, bundle: .module)
                    .foregroundStyle(NymColor.sysOnSurface)
                    .padding(.leading, 8)

                VStack(alignment: .leading) {
                    Text(viewModel.title)
                        .foregroundStyle(NymColor.sysOnSurface)
                        .textStyle(.Body.Large.primary)
                    Text(viewModel.subtitle)
                        .foregroundStyle(NymColor.sysOutline)
                        .textStyle(viewModel.isSmallScreen ? .Body.Small.primary : .Body.Medium.primary)
                }
                .padding(.leading, 8)
                Spacer()
            }
        }
        .frame(height: viewModel.isSmallScreen ? 56 : 64)
        .background(NymColor.navigationBarBackground)
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(viewModel.selectionStrokeColor)
        )
        .animation(.default, value: viewModel.selectionStrokeColor)
    }
}
