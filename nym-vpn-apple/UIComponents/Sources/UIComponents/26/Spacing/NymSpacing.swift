import CoreFoundation

/// Spacing constants drawn from the Nym Figma design system.
///
/// Usage: `.padding(NymSpacing.section)` or `.padding(.horizontal, NymSpacing.component)`
public enum NymSpacing {
    /// 2 pt — micro gap between tightly related elements
    public static let extraExtraSmall: CGFloat = 2
    /// 8 pt — small inset / tight spacing
    public static let small: CGFloat = 8
    /// 12 pt — medium padding
    public static let medium: CGFloat = 12
    /// 15 pt — standard internal margin
    public static let standard: CGFloat = 15
    /// 16 pt — large padding (padding_large token)
    public static let large: CGFloat = 16
    /// 20 pt — outer component margin (home + settings screens)
    public static let component: CGFloat = 20
    /// 24 pt — section-level padding
    public static let section: CGFloat = 24
    /// 480 pt — maximum width for floating drawer cards on wide screens (e.g. macOS)
    public static let drawerMaxWidth: CGFloat = 480
}
