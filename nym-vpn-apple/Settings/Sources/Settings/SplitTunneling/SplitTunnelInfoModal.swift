#if os(macOS)
import SwiftUI
import Theme
import UIComponents

struct SplitTunnelInfoModal: View {
    @Binding var isDisplayed: Bool

    var body: some View {
        ModalOverlayView(isDisplayed: $isDisplayed, dismissOnOverlayTap: false) {
            VStack {
                icon()
                title()
                directSection()
                Spacer()
                    .frame(height: 16)
                viaNymVPNSection()
                gotItButton()
            }
            .padding(.horizontal, 24)
        }
    }
}

private extension SplitTunnelInfoModal {
    @ViewBuilder
    func icon() -> some View {
        Spacer()
            .frame(height: 24)

        Image(systemName: "info.circle")
            .frame(width: 24, height: 24)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text("splitTunnel.info.title".localizedString)
            .nymTextStyle(.titleScreen)
            .foregroundStyle(Color.Nym.textPrimary)

        Spacer()
            .frame(height: 16)

        HStack {
            Text("splitTunnel.info.subtitle".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }

        Spacer()
            .frame(height: 8)

        HStack {
            Text("splitTunnel.info.fullDiskAccess".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func directSection() -> some View {
        HStack(spacing: 8) {
            Image(systemName: "slash.circle.fill")
                .frame(width: 16, height: 16)
                .foregroundStyle(Color.Nym.textSecondary)
            Text("splitTunnel.info.direct".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }

        Spacer()
            .frame(height: 4)

        HStack {
            Text("splitTunnel.info.direct.subtitle".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
    }

    @ViewBuilder
    func viaNymVPNSection() -> some View {
        HStack(spacing: 8) {
            Image(systemName: "shield.fill")
                .frame(width: 16, height: 16)
                .foregroundStyle(Color.Nym.textSecondary)
            Text("splitTunnel.info.viaNymVPN".localizedString)
                .nymTextStyle(.bodyDefault)
                .foregroundStyle(Color.Nym.textPrimary)
            Spacer()
        }

        Spacer()
            .frame(height: 4)

        HStack {
            Text("splitTunnel.info.viaNymVPN.subtitle".localizedString)
                .foregroundStyle(Color.Nym.textSecondary)
                .nymTextStyle(.bodyDefault)
            Spacer()
        }
    }

    @ViewBuilder
    func gotItButton() -> some View {
        GenericButton(title: "splitTunnel.info.gotIt".localizedString)
            .padding(.vertical, 24)
            .onTapGesture {
                isDisplayed.toggle()
            }
    }
}
#endif
