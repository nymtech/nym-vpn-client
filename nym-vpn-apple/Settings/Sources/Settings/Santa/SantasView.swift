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
            santasSpacer()
            VStack {
                enivironmentDetails()
                santasSpacer()
                gatewaySection()
                santasSpacer()
                environmentSection()
            }

            Spacer()
        }
        .preferredColorScheme(AppSettings.shared.currentAppearance.colorScheme)
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
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
                .foregroundStyle(NymColor.primaryOrange)
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

    func gatewaySection() -> some View {
        VStack {
            Text("Gateway-Router:")
                .foregroundStyle(NymColor.primaryOrange)
                .bold()
                .padding(4)

            entryGatewaySection()
            exitGatewaySection()

            Button("Clear both gateways") {
                viewModel.clearBothGateways()
            }
        }
    }

    func entryGatewaySection() -> some View {
        VStack {
            HStack {
                Text("Entry gateway:")
                Spacer()
                Button("Clear") {
                    viewModel.clearEntryGateway()
                }
                Button("Paste") {
                    viewModel.pasteEntryGateway()
                }
            }
            .padding(.horizontal, 16)
            HStack {
                Text("\(viewModel.entryGatewayString())")
                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.bottom, 16)
        }
    }

    func exitGatewaySection() -> some View {
        VStack {
            HStack {
                Text("Exit gateway:")
                Spacer()
                Button("Clear") {
                    viewModel.clearExitGateway()
                }
                Button("Paste") {
                    viewModel.pasteExitGateway()
                }
            }
            .padding(.horizontal, 16)
            HStack {
                Text("\(viewModel.exitGatewayString())")
                Spacer()
            }
            .padding(.horizontal, 16)
        }
    }

    func environmentSection() -> some View {
        VStack {
            Text("Environment:")
                .foregroundStyle(NymColor.primaryOrange)
                .bold()
                .padding(4)

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
