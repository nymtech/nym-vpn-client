import SwiftUI
import Localizations

public struct LocalizedText: View {
    @ObservedObject private var localizationManager: LocalizationManager
    private let key: String

    public init(_ key: String, localizationManager: LocalizationManager = .shared) {
        self.key = key
        self.localizationManager = localizationManager
    }

    public var body: some View {
        Text(localizationManager.localizedString(forKey: key))
    }
}
