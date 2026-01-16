import SwiftUI
import AppSettings
import Constants
import Theme
import UIComponents

public struct SantasView: View {
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
                    environmentDetails()
                    santasSpacer()
                    environmentSection()
                }
                Spacer()
            }
        }
        .preferredColorScheme(AppSettings.shared.currentAppearance.colorScheme)
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
#if os(iOS)
        .alert("Restart Required", isPresented: $viewModel.showRestartAlert) {
            Button("OK") {
                viewModel.showRestartAlert = false
            }
        } message: {
            Text("Please close and restart the app for the environment change to take effect.")
        }
#endif
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

    func environmentDetails() -> some View {
        VStack {
            Text("Environment Details:")
                .foregroundStyle(NymColor.accent)
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
                .foregroundStyle(NymColor.accent)
                .bold()
                .padding(4)
#if os(macOS)
            Text("⚠️ Please restart daemon after switching the env ⚠️")
                .padding(4)
#elseif os(iOS)
            Text("⚠️ Please restart app after switching the env ⚠️")
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
}

private extension SantasView {
    func santasSpacer() -> some View {
        Spacer()
            .frame(height: 16)
    }
}
