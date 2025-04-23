public enum SupportedLanguage: String, CaseIterable, RawRepresentable {
    case english = "en"
    case arabic = "ar"
    case chineeseSimplified = "zh-Hans"
    case french = "fr"
    case german = "de"
    case hindi = "hi"
    case italian = "it"
    case japanese = "ja"
    case persian = "fa"
    case portugues = "pt"
    case portuguesBR = "pt-BR"
    case russian = "ru"
    case spanish = "es"
    case turkish = "tr"
    case ukrainian = "uk"
    case vietnamese = "vi"

    public var localizedName: String {
        switch self {
        case .english:
            "English"
        case .arabic:
            "عربي"
        case .chineeseSimplified:
            "简体中文"
        case .french:
            "Français"
        case .german:
            "Deutsch"
        case .hindi:
            "हन"
        case .italian:
            "Italiano"
        case .japanese:
            "日本語"
        case .persian:
            "فارسی"
        case .portugues:
            "Português"
        case .portuguesBR:
            "Português (Brasil)"
        case .russian:
            "Русский"
        case .spanish:
            "Español"
        case .turkish:
            "Türkçe"
        case .ukrainian:
            "Українська"
        case .vietnamese:
            "Tiếng Việt"
        }
    }
}
