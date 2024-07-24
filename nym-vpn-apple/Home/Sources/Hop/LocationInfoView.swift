import SwiftUI
import ExternalLinkManager
import UIComponents
import Theme

struct LocationInfoView: View {
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
                .background(NymColor.modeInfoViewBackground)
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
            .textStyle(NymTextStyle.Label.Huge.bold)
            .foregroundStyle(NymColor.sysOnSurface)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func message() -> some View {
        HStack {
            Text(viewModel.messageLocalizedString)
                .foregroundStyle(NymColor.modeInfoViewDescription)
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
                .foregroundStyle(NymColor.primaryOrange)

            GenericImage(imageName: viewModel.readMoreLinkImageName)
                .frame(width: 16, height: 16)
                .foregroundStyle(NymColor.primaryOrange)
        }
        .onTapGesture {
            viewModel.openContinueReading()
        }

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
