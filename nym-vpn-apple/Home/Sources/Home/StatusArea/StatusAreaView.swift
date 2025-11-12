import SwiftUI
import AppSettings
import Device
import UIComponents
import TunnelStatus

struct StatusAreaView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding var statusButtonConfig: StatusButtonConfig
    @Binding var statusInfoState: StatusInfoState
    @Binding var connectedDate: Date?
    @Binding var path: NavigationPath
    @Binding var tunnelStatus: TunnelStatus

    var body: some View {
        VStack {
            // TODO: replace !Device.isMacOS with Device.isMacOS after passphrase is supported in macos
            if !appSettings.isSmallScreen,
               !Device.isMacOS,
               tunnelStatus == .connected,
               !appSettings.isPassphraseStored {
                Spacer()
                    .frame(height: 8)
                PassphraseOverlay(path: $path)
                Spacer()
                    .frame(height: 8)
            }
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
