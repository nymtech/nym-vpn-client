import SwiftUI
import UIComponents

/// Centred arc-progress backdrop driven by `ConnectionStatusViewModel`. Sits
/// above the background and below the drawer. Mirrors the source TCA
/// `ConnectionStatusFeatureView`.
struct ConnectionStatusBackdrop: View {
    @State var viewModel: ConnectionStatusViewModel
    var availableHeight: CGFloat?

    var body: some View {
        ArcProgressView(
            state: viewModel.arcProgressState,
            mode: viewModel.mode,
            connectedDate: viewModel.connectedDate,
            showsIndependenceWarning: viewModel.showsIndependenceWarning,
            availableHeight: availableHeight
        )
    }
}
