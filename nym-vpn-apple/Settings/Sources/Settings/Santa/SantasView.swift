import SwiftUI
import AppSettings
import Constants
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
                    togglesSection()
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
#if os(macOS)
            Button("Refetch daemon info") {
                viewModel.updateDaemonInfo()
            }
#endif
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
            HStack {
                ForEach(viewModel.envs, id: \.self) { env in
                    Button(env.rawValue) {
                        viewModel.changeEnvironment(to: env)
                    }
                }
            }
        }
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
