import SwiftUI
import Theme
import UIComponents

public struct QuitAppModal: View {
    private let closeAction: () -> Void
    private let quitAction: () -> Void
    @Binding private var isDisplayed: Bool

    public init(isDisplayed: Binding<Bool>, closeAction: @escaping (() -> Void), quitAction: @escaping (() -> Void)) {
        _isDisplayed = isDisplayed
        self.closeAction = closeAction
        self.quitAction = quitAction
    }

    public var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            VStack {
                Spacer()
                    .frame(height: 24)

                title()
                subtitle()

                closeWindow()
                quitButton()
                cancelButton()

                Spacer()
                    .frame(height: 24)
            }
            .background(NymColor.elevation)
            .cornerRadius(16)
            .padding(32)
        }
        .edgesIgnoringSafeArea(.all)
    }
}

private extension QuitAppModal {
    @ViewBuilder
    func title() -> some View {
        Text("quit.question".localizedString)
            .textStyle(NymTextStyle.Headline.Small.regular)
            .foregroundStyle(NymColor.primary)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func subtitle() -> some View {
        Text("quit.subQuestion".localizedString)
            .textStyle(NymTextStyle.Body.Medium.regular)
            .foregroundStyle(NymColor.gray1)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)

        Spacer()
            .frame(height: 16)
    }

    func closeWindow() -> some View {
        GenericButton(title: "quit.closeWindow".localizedString, height: 39)
            .padding(.horizontal, 24)
            .onTapGesture {
                closeAction()
            }
    }

    func quitButton() -> some View {
        GenericButton(title: "quit.quit".localizedString, height: 39)
            .padding(.horizontal, 24)
            .onTapGesture {
                quitAction()
            }
    }

    func cancelButton() -> some View {
        GenericButton(title: "cancel".localizedString, height: 39)
            .padding(.horizontal, 24)
            .onTapGesture {
                isDisplayed = false
            }
    }
}
