public struct FeatureFlag {
    private let value: String

    public let name: String

    public var isEnabled: Bool {
         value.lowercased() == "true"
    }

    public var rawValue: String {
        value
    }

    public init(name: String, value: String) {
        self.name = name
        self.value = value
    }
}
