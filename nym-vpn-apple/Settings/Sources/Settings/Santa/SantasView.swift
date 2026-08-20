#if SANTA
import SwiftUI
import AppSettings
import Constants
import SnackbarManager
import Theme
import UIComponents

public struct SantasView: View {
    @EnvironmentObject private var appSettings: AppSettings
    @ObservedObject private var viewModel: SantasViewModel

    public init(viewModel: SantasViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack(spacing: .zero) {
            navbar()
            ScrollView {
                santasSpacer()
                VStack {
                    enivironmentDetails()
                    santasSpacer()
                    environmentSection()
                    santasSpacer()
                    deviceSection()
                    santasSpacer()
                    togglesSection()
                    santasSpacer()
                    snackbarSection()
                    santasSpacer()
                    accountSummarySection()
                    santasSpacer()
                    logsSection()
                }
                Spacer()
            }
            .scrollIndicators(.never)
        }
        .preferredColorScheme(AppSettings.shared.currentAppearance.colorScheme)
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
    }
}

private extension SantasView {
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
        .padding(0)
    }

    func enivironmentDetails() -> some View {
        VStack {
            Text("Environment Details:")
                .foregroundStyle(Color.Nym.primary)
                .bold()
                .padding(4)
            Text("App environment: \(viewModel.currentAppEnv)")
                .padding(4)
            Text("Daemon/lib environment: \(viewModel.actualEnv)")
                .padding(4)
            Text("Daemon/lib version: \(viewModel.libVersion)")
                .padding(4)
        }
        .padding(16)
    }

    func environmentSection() -> some View {
        VStack {
            Text("Environment:")
                .foregroundStyle(Color.Nym.primary)
                .bold()
                .padding(4)
#if os(macOS)
            Text("⚠️ Please restart daemon after switching the env ⚠️")
                .padding(4)
#endif
#if os(iOS)
            if viewModel.currentAppEnv != viewModel.actualEnv {
                Text("⚠️ App and network env differ - restart may be required ⚠️")
                    .font(.caption)
                    .padding(4)
            }
            Text(viewModel.storeKitAccountGuidance)
                .font(.caption)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 8)
#endif
            HStack {
                ForEach(viewModel.envs, id: \.self) { env in
                    Button(env.rawValue) {
                        viewModel.changeEnvironment(to: env)
                    }
                }
            }
        }
    }

    func deviceSection() -> some View {
        VStack(spacing: 8) {
            Text("Device register:")
                .foregroundStyle(Color.Nym.primary)
                .bold()
                .padding(4)
            Text("POST /device with the keys already on this phone. Does not log out. Disconnect first. Same Device id in logs means the idempotent API path ran.")
                .font(.caption)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 8)
            Button("Re-register this device") {
                viewModel.reregisterCurrentDevice()
            }
            .disabled(viewModel.isReregisteringDevice)
        }
        .padding(16)
    }

    @ViewBuilder
    func togglesSection() -> some View {
        VStack {
            Toggle(isOn: $appSettings.isMixnetTuningEnabled) {
                Text("Mixnet tuning")
            }
            .tint(Color.Nym.primary)
        }.padding()
    }

    func snackbarSection() -> some View {
        VStack(spacing: 8) {
            Text("Snackbar tests:")
                .foregroundStyle(Color.Nym.primary)
                .bold()
                .padding(4)
            Text("Tap, then pop back to Home to see them play.")
                .font(.caption)
                .padding(4)

            Text("Style coverage")
                .foregroundStyle(Color.Nym.primary)
                .padding(.top, 4)
            Button("Queue all 5 styles") {
                SantasView.styleFixtures.forEach { SnackbarManager.shared.enqueue($0.item) }
            }
            ForEach(SantasView.styleFixtures, id: \.label) { fixture in
                Button(fixture.label) {
                    SnackbarManager.shared.enqueue(fixture.item)
                }
            }

            Text("Real scenarios")
                .foregroundStyle(Color.Nym.primary)
                .padding(.top, 8)
            Button("Queue all real scenarios") {
                SantasView.realFixtures.forEach { SnackbarManager.shared.enqueue($0.item) }
            }
            ForEach(SantasView.realFixtures, id: \.label) { fixture in
                Button(fixture.label) {
                    SnackbarManager.shared.enqueue(fixture.item)
                }
            }

            Button("Clear queue") {
                SnackbarManager.shared.clear()
            }
            .padding(.top, 8)
        }
        .padding(16)
    }

    func accountSummarySection() -> some View {
        VStack(spacing: 8) {
            Text("Account summary fakes:")
                .foregroundStyle(Color.Nym.primary)
                .bold()
                .padding(4)
            Text("Fakes subscription expiry. Check Settings + Account & Devices.")
                .font(.caption)
                .padding(4)

            Toggle(isOn: $viewModel.fakeAutoRenew) {
                Text("isAutoRenewEnabled")
            }
            .tint(Color.Nym.primary)
            .padding(.horizontal, 4)

            Text("Yearly plan")
                .foregroundStyle(Color.Nym.primary)
                .padding(.top, 4)
            presetRow(viewModel.yearlyPresets)

            Text("Monthly plan")
                .foregroundStyle(Color.Nym.primary)
                .padding(.top, 4)
            presetRow(viewModel.monthlyPresets)

            Button("Clear override") {
                viewModel.clearAccountSummaryOverride()
            }
            .padding(.top, 8)
        }
        .padding(16)
    }

    func presetRow(_ presets: [SantasViewModel.AccountSummaryPreset]) -> some View {
        ScrollView(.horizontal, showsIndicators: false) {
            HStack(spacing: 8) {
                ForEach(presets, id: \.label) { preset in
                    Button(preset.label) {
                        viewModel.applyAccountSummaryPreset(preset)
                    }
                }
            }
            .padding(.horizontal, 4)
        }
    }

    func logsSection() -> some View {
        VStack {
            Text("Logs:")
                .foregroundStyle(Color.Nym.primary)
                .bold()
                .padding(4)
            Text("Logs size: \(viewModel.logFilesSize)")
                .padding(4)
        }
        .padding(16)
    }
}

private extension SantasView {
    func santasSpacer() -> some View {
        Spacer()
            .frame(height: 16)
    }
}

private extension SantasView {
    struct SnackbarFixture {
        let label: String
        let item: SnackbarItem
    }

    static let styleFixtures: [SnackbarFixture] = [
        SnackbarFixture(
            label: "Critical (action)",
            item: SnackbarItem(
                style: .critical,
                title: "Error connecting",
                message: "The selected gateway is not available!",
                actionTitle: "Try again",
                onAction: {}
            )
        ),
        SnackbarFixture(
            label: "Confirmation",
            item: SnackbarItem(
                style: .confirmation,
                title: "Renewal success!",
                message: "Welcome back to actual privacy."
            )
        ),
        SnackbarFixture(
            label: "Neutral (action)",
            item: SnackbarItem(
                style: .neutral,
                title: "Heads up",
                message: "Regular info",
                actionTitle: "Action",
                onAction: {}
            )
        ),
        SnackbarFixture(
            label: "Negative (action)",
            item: SnackbarItem(
                style: .negative,
                title: "Negative alert",
                message: "Explain negative situation",
                actionTitle: "Action",
                onAction: {}
            )
        ),
        SnackbarFixture(
            label: "Warning",
            item: SnackbarItem(
                style: .warning,
                title: "Subscription expired"
            )
        )
    ]

    static var realFixtures: [SnackbarFixture] {
        [
            SnackbarFixture(
                label: "OneClick: offline",
                item: SnackbarItem(
                    style: .warning,
                    title: "home.modal.noInternetConnection.title".localizedString,
                    message: "home.modal.noInternetConnection.subtitle".localizedString
                )
            ),
            SnackbarFixture(
                label: "OneClick: connection failed (retry)",
                item: SnackbarItem(
                    style: .critical,
                    title: "connectionError.title".localizedString,
                    message: connectionErrorBody(
                        reason: "Mock failure: gateway timed out."
                    ),
                    actionTitle: "disconnect".localizedString,
                    onAction: {},
                    duration: 7
                )
            ),
            SnackbarFixture(
                label: "Auth: login failed",
                item: SnackbarItem(
                    style: .critical,
                    title: "error".localizedString,
                    message: "Invalid recovery phrase."
                )
            ),
            SnackbarFixture(
                label: "Proxy: enabled",
                item: SnackbarItem(
                    style: .confirmation,
                    title: "proxy.snackbar.successfullyEnabled".localizedString
                )
            ),
            SnackbarFixture(
                label: "Proxy: connection failed",
                item: SnackbarItem(
                    style: .negative,
                    title: "proxy.snackbar.connectionFailed".localizedString
                )
            ),
            SnackbarFixture(
                label: "DNS: saved",
                item: SnackbarItem(
                    style: .confirmation,
                    title: "dns.snackbar.saved".localizedString
                )
            ),
            SnackbarFixture(
                label: "Generic: something went wrong",
                item: SnackbarItem(
                    style: .negative,
                    title: "generalNymError.somethingWentWrong".localizedString
                )
            ),
            SnackbarFixture(
                label: "Mixnet tuning: saved",
                item: SnackbarItem(
                    style: .confirmation,
                    title: "mixnetTuning.snackbar.saved".localizedString
                )
            ),
            SnackbarFixture(
                label: "Copied to pasteboard",
                item: SnackbarItem(
                    style: .confirmation,
                    title: "settings.copiedToPasteboard".localizedString
                )
            )
        ]
    }

    /// Mirrors `Home/Sources/26/ConnectionStatus/ConnectionErrorCopy.message(reason:)`.
    /// Inlined because ConnectionErrorCopy is internal to the Home module.
    static func connectionErrorBody(reason: String?) -> String {
        let hint = "connectionError.killswitchHint".localizedString
        let instruction = "connectionError.disconnectInstruction".localizedString
        let tail = hint + "\n\n" + instruction
        guard let reason, !reason.isEmpty else { return tail }
        return reason + "\n\n" + tail
    }
}
#endif
