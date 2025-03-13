import SwiftUI
import AppSettings
import ConnectionManager
import UIComponents

struct StatusAreaView: View {
    @EnvironmentObject private var connectionManager: ConnectionManager

    @Binding var statusButtonConfig: StatusButtonConfig
    @Binding var statusInfoState: StatusInfoState

    var body: some View {
        VStack {
            NoiseConnectedAnimationView()
            Spacer()
                .frame(height: 8)

            StatusButton(config: statusButtonConfig)
            Spacer()
                .frame(height: 8)

            StatusInfoView(
                timeConnectedString: $connectionManager.connectedDateString,
                infoState: $statusInfoState
            )
        }
    }
}
