import SwiftUI
import ExternalLinkManager
import UIComponents
import Theme

struct ModeSelectionInfoView: View {
    private let viewModel: ModeSelectionInfoViewModel

    init(viewModel: ModeSelectionInfoViewModel) {
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
                    anonymousTitle()
                    anonymousDescription()
                    fastTitle()
                    fastDescription()
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

private extension ModeSelectionInfoView {
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
    func anonymousTitle() -> some View {
        HStack {
            GenericImage(imageName: viewModel.anonymousImageName)
                .foregroundStyle(NymColor.modeInfoViewTitle)
                .frame(width: 16, height: 16)
                .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 8))

            Text(viewModel.anonymousTitleLocalizedString)
                .textStyle(.Label.Large.bold)
                .foregroundStyle(NymColor.modeInfoViewTitle)

            Spacer()
        }
        Spacer()
            .frame(height: 8)
    }

    @ViewBuilder
    func anonymousDescription() -> some View {
        HStack {
            Text(viewModel.anonymousDescriptionLocalizedString)
                .foregroundStyle(NymColor.modeInfoViewDescription)
                .textStyle(.Body.Medium.regular)

            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 24))

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func fastTitle() -> some View {
        HStack {
            GenericImage(imageName: viewModel.fastImageName)
                .foregroundStyle(NymColor.modeInfoViewTitle)
                .frame(width: 16, height: 16)
                .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 8))

            Text(viewModel.fastTitleLocalizedString)
                .textStyle(.Label.Large.bold)
                .foregroundStyle(NymColor.modeInfoViewTitle)

            Spacer()
        }
        Spacer()
            .frame(height: 8)
    }

    @ViewBuilder
    func fastDescription() -> some View {
        HStack {
            Text(viewModel.fastDescriptionLocalizedString)
                .foregroundStyle(NymColor.modeInfoViewDescription)
                .textStyle(.Body.Medium.regular)

            Spacer()
        }
        .padding(EdgeInsets(top: 0, leading: 24, bottom: 0, trailing: 24))

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func continueReadingLink() -> some View {
        HStack {
            Text(viewModel.continueReadingLocalizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primaryOrange)

            GenericImage(imageName: viewModel.continueReadingLinkImageName)
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
                withAnimation {
                    viewModel.isDisplayed.toggle()
                }
            }
    }
}
