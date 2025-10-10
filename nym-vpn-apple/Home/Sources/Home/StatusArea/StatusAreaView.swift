import SwiftUI
import UIComponents

struct StatusAreaView: View {
    @Binding var statusButtonConfig: StatusButtonConfig
    @Binding var statusInfoState: StatusInfoState
    @Binding var connectedDate: Date?

    var body: some View {
        VStack {
            NoiseConnectedAnimationView()
            Spacer()
                .frame(height: 8)

            StatusButton(config: statusButtonConfig)
            Spacer()
                .frame(height: 8)

            StatusInfoView(
                connectedDate: $connectedDate,
                infoState: $statusInfoState
            )
        }
    }
}
