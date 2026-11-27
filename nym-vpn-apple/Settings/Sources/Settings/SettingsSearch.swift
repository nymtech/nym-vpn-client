import Foundation
import UIComponents

/// Filters settings rows by title, keeping each match under its own section.
enum SettingsSearch {
    /// `rowlessTitle` names what a section with no rows draws instead (the signed-out
    /// Get started button), so a search for that title keeps the section.
    static func filter<Kind: SettingsSectionKind>(
        _ sections: [SettingsSection<Kind>],
        query: String,
        rowlessTitle: (Kind) -> String? = { _ in nil }
    ) -> [SettingsSection<Kind>] {
        let query = query.trimmingCharacters(in: .whitespaces)
        guard !query.isEmpty else { return sections }
        func matches(_ title: String) -> Bool {
            title.range(of: query, options: [.caseInsensitive, .diacriticInsensitive], locale: .current) != nil
        }
        return sections.compactMap { section in
            if section.viewModels.isEmpty, let title = rowlessTitle(section.kind) {
                return matches(title) ? section : nil
            }
            let hits = section.viewModels.filter { matches($0.title) }
            return hits.isEmpty ? nil : SettingsSection(kind: section.kind, viewModels: hits)
        }
    }
}
