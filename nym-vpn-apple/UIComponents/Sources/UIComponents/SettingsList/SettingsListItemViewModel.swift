import Foundation
import SwiftUI
import Theme

public final class SettingsListItemViewModel: ObservableObject, Hashable {
    public enum Accessory: Hashable {
        case arrow
        case toggle(isOn: Binding<Bool>, isDisabled: Bool = false)
        case externalLink
        case copy
        case empty

        public func hash(into hasher: inout Hasher) {
            switch self {
            case .arrow:
                hasher.combine(0)
            case let .toggle(_, isDisabled):
                hasher.combine(1)
                hasher.combine(isDisabled)
            case .externalLink:
                hasher.combine(2)
            case .copy:
                hasher.combine(3)
            case .empty:
                hasher.combine(4)
            }
        }

        public static func == (lhs: Accessory, rhs: Accessory) -> Bool {
            switch (lhs, rhs) {
            case (.arrow, .arrow), (.externalLink, .externalLink), (.copy, .copy), (.empty, .empty):
                true
            case let (.toggle(_, lhsDisabled), .toggle(_, rhsDisabled)):
                lhsDisabled == rhsDisabled
            default:
                false
            }
        }

        var imageName: String? {
            switch self {
            case .arrow:
                "arrowRight"
            case .externalLink:
                "externalLink"
            case .copy:
                "copy"
            case .toggle, .empty:
                nil
            }
        }

        var imageColor: Color {
            switch self {
            case .arrow, .toggle, .copy:
                NymColor.primary
            case .externalLink:
                NymColor.gray1
            case .empty:
                .clear
            }
        }

        public var isToggle: Bool {
            guard case .toggle = self else { return false }
            return true
        }

        var accessibilityHint: String {
            switch self {
            case .toggle:
                "accessibility.doubleTap.toggle".localizedString
            case .externalLink:
                "accessibility.doubleTap.externalLink".localizedString
            case .copy:
                "accessibility.doubleTap.copy".localizedString
            case .arrow, .empty:
                ""
            }
        }

        var accessibilityValue: String {
            switch self {
            case let .toggle(isOn, isDisabled):
                let value = isOn.wrappedValue ? "general.on".localizedString : "general.off".localizedString
                if isDisabled {
                    return "\(value) \("accessibility.dimmed".localizedString)"
                } else {
                    return value
                }
            case .arrow, .externalLink, .copy, .empty:
                return ""
            }
        }
    }

    public enum ItemType {
        case regular
        case destructive

        public var backgroundColor: Color {
            switch self {
            case .regular:
                NymColor.elevation
            case .destructive:
                NymColor.error.opacity(0.1)
            }
        }

        public var strokeColor: Color {
            switch self {
            case .regular:
                NymColor.elevation
            case .destructive:
                NymColor.error
            }
        }
    }

    let title: String
    let titleTextStyle: NymTextStyle
    let multilineText: AttributedString?
    let imageName: String?
    let systemImageName: String?
    public let type: ItemType
    let isHoveredHighlightDisabled: Bool
    let accessory: Accessory
    let action: (() -> Void)

    @Published public var subtitle: AttributedString?
    var position: SettingsListItemPosition

    public init(
        accessory: Accessory,
        title: String,
        titleTextStyle: NymTextStyle = .Body.Large.regular,
        subtitle: String? = nil,
        attributtedSubtitle: AttributedString? = nil,
        multilineText: AttributedString? = nil,
        imageName: String? = nil,
        systemImageName: String? = nil,
        type: ItemType = .regular,
        isHoveredHighlightDisabled: Bool = false,
        position: SettingsListItemPosition = SettingsListItemPosition(isFirst: false, isLast: false),
        action: @escaping (() -> Void)
    ) {
        self.title = title
        self.titleTextStyle = titleTextStyle
        if let subtitle {
            self.subtitle = AttributedString(subtitle)
        } else if let attributtedSubtitle {
            self.subtitle = attributtedSubtitle
        } else {
            self.subtitle = nil
        }
        self.multilineText = multilineText
        self.imageName = imageName
        self.systemImageName = systemImageName
        self.type = type
        self.accessory = accessory
        self.isHoveredHighlightDisabled = isHoveredHighlightDisabled
        self.position = position
        self.action = action
    }

    public var topRadius: CGFloat {
        if position.isFirst {
            return CGFloat(8)
        } else {
            return CGFloat(0)
        }
    }

    public var bottomRadius: CGFloat {
        if position.isLast {
            return CGFloat(8)
        } else {
            return CGFloat(0)
        }
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(title)
        hasher.combine(subtitle)
        hasher.combine(imageName)
        hasher.combine(accessory)
    }

    public static func == (lhs: SettingsListItemViewModel, rhs: SettingsListItemViewModel) -> Bool {
        lhs.hashValue == rhs.hashValue
    }
}

public struct SettingsListItemPosition: Hashable {
    public var isFirst: Bool
    public var isLast: Bool

    public init(isFirst: Bool, isLast: Bool) {
        self.isFirst = isFirst
        self.isLast = isLast
    }
}
