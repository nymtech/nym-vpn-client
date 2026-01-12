import SwiftUI
import AppSettings
import Device
import Routes
import Settings
import Theme
import UIComponents

struct PassphraseOverlay: View {
    @EnvironmentObject private var appSettings: AppSettings
    @Binding private var path: NavigationPath

    var body: some View {
        ZStack {
            VStack(alignment: .center, spacing: 0) {
                if appSettings.isSmallScreen && !Device.isMacOS {
                    Spacer()
                        .frame(height: 55)
                }
                content
                Spacer()
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            .padding(appSettings.isSmallScreen && !Device.isMacOS ? 16 : 0)
        }
        .edgesIgnoringSafeArea(.all)
    }

    init(path: Binding<NavigationPath>) {
        _path = path
    }
}

private extension PassphraseOverlay {
    var titleText: AttributedString {
        let first = AttributedString("passphraseOverlay.connected".localizedString)
        let second = AttributedString("passphraseOverlay.secureAccess".localizedString)
        return first + AttributedString("\n") + second + " 🔒"
    }

    var content: some View {
        HStack(spacing: 8) {
            VStack(spacing: 0) {
                Text(titleText)
                    .foregroundStyle(NymColor.primary)
                    .textStyle(.Body.Medium.regular)
                    .layoutPriority(2)
            }
            Spacer()
            Text("passphraseOverlay.backup".localizedString)
                .foregroundStyle(NymColor.accent)
                .textStyle(.Body.Medium.bold)
                .contentShape(Rectangle())
                .layoutPriority(1)
                .onTapGesture {
                    navigateToPassphrase()
                }
                .accessibilityAction {
                    navigateToPassphrase()
                }
        }
        .frame(maxWidth: .infinity)
        .padding(16)
        .background(NymColor.elevation)
        .cornerRadius(4)
    }
}

private extension PassphraseOverlay {
    func navigateToPassphrase() {
        path.append(HomeLink.settings)
        path.append(SettingLink.passphrase)
    }
}
