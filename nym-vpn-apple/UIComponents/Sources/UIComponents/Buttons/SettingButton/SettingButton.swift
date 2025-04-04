import SwiftUI
import Theme

public struct SettingButton: View {
    private let viewModel: SettingButtonViewModel

    @Binding private var isHovered: Bool

    public init(viewModel: SettingButtonViewModel, isHovered: Binding<Bool>) {
        self.viewModel = viewModel
        _isHovered = isHovered
    }

    public var body: some View {
        VStack {
            HStack {
                Image(viewModel.selectionImageName, bundle: .module)
                    .foregroundStyle(viewModel.selectionImageColor)
                    .padding(.leading, 16)

                VStack(alignment: .leading) {
                    Text(viewModel.title)
                        .foregroundStyle(NymColor.primary)
                        .textStyle(.Body.Large.regular)
                    if let subtitle = viewModel.subtitle {
                        Text(subtitle)
                            .foregroundStyle(NymColor.gray1)
                            .textStyle(.Body.Medium.regular)
                    }
                }
                .padding(.leading, 8)
                Spacer()
            }
        }
        .frame(maxWidth: .infinity, minHeight: 64, maxHeight: 64)
        .background(NymColor.elevation.opacity(isHovered ? 0.7 : 1))
        .cornerRadius(8)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .inset(by: 0.5)
                .stroke(viewModel.selectionStrokeColor)
        )
        .animation(.default, value: viewModel.selectionStrokeColor)
    }
}
