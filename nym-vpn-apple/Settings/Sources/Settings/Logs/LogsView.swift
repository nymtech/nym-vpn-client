import SwiftUI
import Theme
import UIComponents

public struct LogsView: View {
    @State private var hapticTrigger = 0
    private let viewModel: LogsViewModel

    public init(viewModel: LogsViewModel) {
        self.viewModel = viewModel
    }

    public var body: some View {
        VStack {
            navbar()
            Spacer()

            ScrollView {
                Text(viewModel.logs)
                    .padding()
            }

            buttonsSection()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

private extension LogsView {
    @ViewBuilder
    func navbar() -> some View {
        CustomNavBar(
            title: viewModel.title,
            leftButton: CustomNavBarButton(type: .back, action: { viewModel.navigateBack() })
        )
    }

    @ViewBuilder
    func copyButton() -> some View {
        GenericButton(title: viewModel.copyLocalizedString)
            .padding(16)
            .onTapGesture {
                viewModel.copyToPasteBoard()
                hapticTrigger += 1
            }
    }

    @ViewBuilder
    func deleteButton() -> some View {
        GenericButton(title: viewModel.deleteLocalizedString)
            .padding(16)
            .onTapGesture {
                viewModel.deleteLogs()
                hapticTrigger += 1
            }
    }

    @ViewBuilder
    func buttonsSection() -> some View {
        if #available(iOS 17.0, *), #available(macOS 14.0, *) {
            HStack {
                copyButton()
                    .sensoryFeedback(.success, trigger: hapticTrigger)
                deleteButton()
                    .sensoryFeedback(.success, trigger: hapticTrigger)
            }
        } else {
            HStack {
                copyButton()
                deleteButton()
            }
        }
    }
}
