import Foundation
import UIComponents

struct SettingsSearchSection: Hashable {
    let kind: AppSettingsSectionKind
    let rows: [SettingsListItemViewModel]
    let blocks: [SettingsBlock]
}

enum SettingsSearch {
    /// Matching rows and blocks per section, in display order; a section title match keeps the whole section.
    static func filter(
        _ sections: [SettingsSection<AppSettingsSectionKind>],
        query: String,
        blocks: (AppSettingsSectionKind) -> [SettingsBlock] = { _ in [] }
    ) -> [SettingsSearchSection] {
        let query = query.trimmingCharacters(in: .whitespaces)
        func matches(_ text: String?) -> Bool {
            guard let text else { return false }
            return text.range(of: query, options: [.caseInsensitive, .diacriticInsensitive], locale: .current) != nil
        }
        return sections.compactMap { section in
            let sectionBlocks = blocks(section.kind)
            let result: SettingsSearchSection
            if query.isEmpty || matches(section.kind.title) {
                result = SettingsSearchSection(kind: section.kind, rows: section.viewModels, blocks: sectionBlocks)
            } else {
                result = SettingsSearchSection(
                    kind: section.kind,
                    rows: section.viewModels.filter { row in
                        matches(row.title) || matches(row.subtitle.map { String($0.characters) })
                    },
                    blocks: sectionBlocks.filter { matches($0.title) }
                )
            }
            return result.rows.isEmpty && result.blocks.isEmpty ? nil : result
        }
    }
}
