import SwiftUI
import Theme

struct ToggleView: View {
    @ObservedObject var viewModel: ToggleViewModel

    var body: some View {
        RoundedRectangle(cornerRadius: 100)
            .inset(by: 1)
            .stroke(viewModel.strokeColor, lineWidth: 2)
            .frame(width: 52, height: 32)
            .background(viewModel.backgroundColor)
            .clipShape(RoundedRectangle(cornerRadius: 100))
            .overlay {
                Circle()
                    .frame(width: viewModel.circleDiameter, height: viewModel.circleDiameter)
                    .foregroundStyle(viewModel.circleColor)
                    .offset(x: viewModel.offset)
                    .animation(.smooth, value: viewModel.offset)
                    .animation(.default, value: viewModel.circleColor)
                    .animation(.default, value: viewModel.backgroundColor)
                    .animation(.default, value: viewModel.strokeColor)
            }
            .onTapGesture {
                viewModel.onTap()
            }
    }
}

#Preview {
    ToggleView(viewModel: ToggleViewModel(isOn: true, action: { _ in }))
}
