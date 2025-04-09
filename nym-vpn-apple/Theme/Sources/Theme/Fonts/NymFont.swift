import SwiftUI

public enum NymFont {
    case labGrotesque(size: CGFloat, weight: LabGrotesqueWeight)
    case labGrotesqueMono(size: CGFloat, weight: LabGrotesqueMonoWeight)

    public var font: Font {
        switch self {
        case let .labGrotesque(size: size, weight: weight):
            Font.custom("LabGrotesque-\(weight.rawValue)", size: size)
        case let .labGrotesqueMono(size: size, weight: weight):
            Font.custom("LabGrotesqueMono-\(weight.rawValue)", size: size)
        }
    }
}

// MARK: - Weights -

extension NymFont {
    public enum LabGrotesqueWeight: String, CaseIterable {
        case regular = "Regular"
    }

    public enum LabGrotesqueMonoWeight: String, CaseIterable {
        case regular = "Regular"
        case bold = "Bold"
    }
}

// MARK: - Register fonts -

extension NymFont {
    public static func register() {
        NymFont.LabGrotesqueWeight.allCases.forEach { latoWeight in
            let fontName = "LabGrotesque-\(latoWeight.rawValue)"
            guard let fontURL = Bundle.module.url(forResource: fontName, withExtension: "ttf") else { return }
            CTFontManagerRegisterFontsForURL(fontURL as CFURL, .process, nil)
        }
        NymFont.LabGrotesqueMonoWeight.allCases.forEach { latoWeight in
            let fontName = "LabGrotesqueMono-\(latoWeight.rawValue)"
            guard let fontURL = Bundle.module.url(forResource: fontName, withExtension: "ttf") else { return }
            CTFontManagerRegisterFontsForURL(fontURL as CFURL, .process, nil)
        }
    }
}
