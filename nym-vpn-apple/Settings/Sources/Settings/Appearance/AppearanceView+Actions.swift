extension AppearanceView {
    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func navigateToDisplayTheme() {
        path.append(SettingLink.displayTheme)
    }

    func navigateToLanguage() {
        path.append(SettingLink.language)
    }
}
