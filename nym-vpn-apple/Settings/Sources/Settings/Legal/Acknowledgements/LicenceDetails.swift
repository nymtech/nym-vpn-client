import Foundation

public final class LicenceDetails: Hashable {
    let title: String
    var text: String?
    var license: String?
    var repository: URL?

    public init(
        title: String,
        text: String? = nil,
        license: String? = nil,
        repository: URL? = nil
    ) {
        self.title = title
        self.text = text
        self.license = license
        self.repository = repository
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(title)
        hasher.combine(text)
        hasher.combine(license)
        hasher.combine(repository)
    }
}

extension LicenceDetails: Equatable {
    public static func == (lhs: LicenceDetails, rhs: LicenceDetails) -> Bool {
        lhs.title == rhs.title && lhs.text == rhs.text && lhs.license == rhs.license && lhs.repository == rhs.repository
    }
}
