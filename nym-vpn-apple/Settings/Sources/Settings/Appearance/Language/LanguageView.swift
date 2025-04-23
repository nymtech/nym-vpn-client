import SwiftUI
import Localizations
import Theme
import UIComponents

public struct LanguageView: View {
    @EnvironmentObject private var localizationManager: LocalizationManager

    @Binding var path: NavigationPath

    public init(path: Binding<NavigationPath>) {
        _path = path
    }

    public var body: some View {
        VStack(spacing: 0) {
            navbar()
            Spacer()
                .frame(height: 24)
            ScrollView {
                automaticLanguage()
                languagesList()
            }
            .frame(maxWidth: MagicNumbers.maxWidth)
            Spacer()
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }
}

// MARK: - Views -
extension LanguageView {
    func navbar() -> some View {
        CustomNavBar(
            title: "settings.language".localizedString,
            leftButton: CustomNavBarButton(type: .back, action: { navigateBack() })
        )
    }

    @ViewBuilder
    func automaticLanguage() -> some View {
        HStack {
            LocalizedText("language.automatic")
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
            Spacer()
        }
        .padding(EdgeInsets(top: 20, leading: 16, bottom: 20, trailing: 16))
        .onTapGesture {
            localizationManager.language = ""
            navigateBack()
        }
    }

    @ViewBuilder
    func languagesList() -> some View {
        ForEach(SupportedLanguage.allCases, id: \.self) { language in
            languageCell(with: language)
        }
    }
}

// MARK: - UI Components -
private extension LanguageView {
    @ViewBuilder
    func languageCell(with language: SupportedLanguage) -> some View {
        HStack {
            Text(language.localizedName)
                .foregroundStyle(NymColor.primary)
                .textStyle(.Body.Large.regular)
            Spacer()
        }
        .padding(EdgeInsets(top: 20, leading: 16, bottom: 20, trailing: 16))
        .onTapGesture {
            localizationManager.language = language.rawValue
            navigateBack()
        }
    }
}

// MARK: - Actions -
private extension LanguageView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }
}
