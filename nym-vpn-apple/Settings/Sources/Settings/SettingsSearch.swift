import Foundation
import UIComponents

/// Filters settings rows by title, keeping each match under its own section.
enum SettingsSearch {
    static func filter<Kind: SettingsSectionKind>(
        _ sections: [SettingsSection<Kind>],
        query: String
    ) -> [SettingsSection<Kind>] {
        let query = query.trimmingCharacters(in: .whitespaces)
        guard !query.isEmpty else { return sections }
        return sections.compactMap { section in
            let hits = section.viewModels.filter {
                $0.title.range(of: query, options: [.caseInsensitive, .diacriticInsensitive], locale: .current) != nil
            }
            return hits.isEmpty ? nil : SettingsSection(kind: section.kind, viewModels: hits)
        }
    }
}
