import SwiftUI
import Theme

public struct SurveyButton: View {
    @ObservedObject private var viewModel: SurveyButtonViewModel

    public init(viewModel: SurveyButtonViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            HStack {
                Image(viewModel.selectionImageName, bundle: .module)
                    .foregroundStyle(viewModel.selectionImageColor)
                    .padding(.leading, 16)

                Text(viewModel.type.title)
                    .foregroundStyle(NymColor.sysOnSurface)
                    .textStyle(.Body.Large.primary)
                    .padding(.leading, 8)
                Spacer()
            }
        }
        .frame(height: 64)
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
