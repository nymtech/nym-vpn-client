#if os(iOS)
import NymVPNLib
import FeatureFlagModels

extension FeatureFlags {
    public func toFeatureFlagList() -> [FeatureFlag] {
        flags.flatMap { key, value in
            switch value {
            case let .value(flagValue):
                return [FeatureFlag(name: key, value: flagValue)]

            case let .group(groupValues):
                return groupValues.map { subKey, subValue in
                    FeatureFlag(name: "\(key).\(subKey)", value: subValue)
                }
            }
        }
    }
}
#endif
