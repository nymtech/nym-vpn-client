import SwiftUI
import ExternalLinkManager
import UIComponents
import Theme

struct LocationInfoView: View {
    @State private var isContinueReadingLinkHovered = false

    private let viewModel: LocationInfoViewModel

    init(viewModel: LocationInfoViewModel) {
        self.viewModel = viewModel
    }

    var body: some View {
        ZStack {
            Rectangle()
                .foregroundColor(.black)
                .opacity(0.3)
                .background(Color.clear)
                .contentShape(Rectangle())

            HStack {
                Spacer()
                    .frame(width: 40)

                VStack {
                    icon()
                    title()
                    message()
                    continueReadingLink()
                    okButton()
                }
                .background(NymColor.elevation)
                .cornerRadius(16)

                Spacer()
                    .frame(width: 40)
            }
        }
        .edgesIgnoringSafeArea(.all)
    }
}

private extension LocationInfoView {
    @ViewBuilder
    func icon() -> some View {
        Spacer()
            .frame(height: 24)

        Image(systemName: viewModel.infoIconImageName)
            .frame(width: 24, height: 24)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text(viewModel.titleLocalizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func message() -> some View {
        HStack {
            Text(viewModel.messageLocalizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
                .multilineTextAlignment(.center)

            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 24))

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func continueReadingLink() -> some View {
        HStack {
            Text(viewModel.readMoreLocalizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.accent)

            GenericImage(imageName: viewModel.readMoreLinkImageName)
                .frame(width: 16, height: 16)
                .foregroundStyle(NymColor.accent)
        }
        .onTapGesture {
            viewModel.openContinueReading()
        }
        .onHover { newValue in
            isContinueReadingLinkHovered = newValue
        }
        .opacity(isContinueReadingLinkHovered ? 0.7 : 1)

        Spacer()
            .frame(height: 24)
    }

    @ViewBuilder
    func okButton() -> some View {
        GenericButton(title: viewModel.okLocalizedString)
            .padding(EdgeInsets(top: 0, leading: 24, bottom: 24, trailing: 24))
            .onTapGesture {
                viewModel.isDisplayed.toggle()
            }
    }
}
