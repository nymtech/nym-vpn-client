import SwiftUI
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

public extension Color {
    enum Nym {
        // MARK: Brand
        /// Spring Green — primary action / accent
        public static let primary      = Color("Nym.Primary", bundle: .module)      // #5BF0A0
        /// Primary hover state
        public static let primaryHover = Color("Nym.PrimaryHover", bundle: .module) // #4AD88C
        /// Primary dark — gradient endpoint
        public static let primaryDark  = Color("Nym.PrimaryDark", bundle: .module)  // #076B3E

        // MARK: Backgrounds
        /// Deepest background — Aztec
        public static let backgroundDeep     = Color("Nym.BackgroundDeep", bundle: .module)     // L #E8F2F0  D #091312
        /// Default background — Woodsmoke
        public static let background         = Color("Nym.Background", bundle: .module)         // L #F7F7FA  D #000000
        /// Elevated surface — Shark
        public static let backgroundElevated = Color("Nym.BackgroundElevated", bundle: .module) // L #F2F2F7  D #18181F
        /// Hover/pressed surface
        public static let backgroundHover    = Color("Nym.BackgroundHover", bundle: .module)    // L #EBEBF0  D #211F2A
        /// Card / inset surface — Baltic Sea
        public static let backgroundCard     = Color("Nym.BackgroundCard", bundle: .module)     // L #FFFFFF  D #211F28
        /// Top nav bar surface
        public static let navBarBackground   = Color("Nym.NavBarBackground", bundle: .module)   // L #FFFFFF  D #1E1E1E

        // MARK: Text
        /// Primary text on dark backgrounds
        public static let textPrimary   = Color("Nym.TextPrimary", bundle: .module)   // L #111116  D #FFFFFF
        /// Secondary text
        public static let textSecondary = Color("Nym.TextSecondary", bundle: .module) // L #52525F  D #AEACB1
        /// Tertiary text
        public static let textTertiary  = Color("Nym.TextTertiary", bundle: .module)  // L #8A8A9A  D #8B8B90
        /// Disabled text
        public static let textDisabled  = Color("Nym.TextDisabled", bundle: .module)  // #6C6C6F

        // MARK: Semantic
        /// Error
        public static let error   = Color("Nym.Error", bundle: .module)   // #E73E14
        /// Warning
        public static let warning = Color("Nym.Warning", bundle: .module) // #FFB400
        /// Success
        public static let success = Color("Nym.Success", bundle: .module) // #28C96C
        /// Info
        public static let info    = Color("Nym.Info", bundle: .module)    // #485ECA

        // MARK: Brand accent colors
        /// Orange — used for "expiring soon" urgency (matches `NymColor.orange` in legacy Theme)
        public static let orange = Color(red: 0.98, green: 0.43, blue: 0.31)          // #FA6E4F

        // MARK: UI Chrome
        /// Divider / separator line
        public static let divider = Color("Nym.Divider", bundle: .module) // L #D1D1D8  D #39393B
        /// Mid-gray — used for subtle icons and inactive carets
        public static let gray1   = Color("Nym.Gray1", bundle: .module)   // #B0ADB6
        /// Darker mid-gray — used for connected button border
        public static let gray2   = Color("Nym.Gray2", bundle: .module)   // #66656A
        /// Very dark near-black gray — anonymous mode thumb in dark
        public static let gray12  = Color("Nym.Gray12", bundle: .module)  // #1D1D1F
        /// Icon default
        public static let icon    = Color("Nym.Icon", bundle: .module)    // #5F6268
        /// Pure black
        public static let black   = Color("Nym.Black", bundle: .module)   // #000000
        /// Pure white
        public static let white   = Color("Nym.White", bundle: .module)   // #FFFFFF

        // MARK: Transparent overlays
        /// White 6% overlay — ring stroke
        public static let white6    = Color("Nym.White6", bundle: .module)    // #FFFFFF @ 6%
        /// White 8% overlay
        public static let white8    = Color("Nym.White8", bundle: .module)    // #FFFFFF @ 8%
        /// Primary 8% overlay — subtle tint
        public static let primary8  = Color("Nym.Primary8", bundle: .module)  // #5BF0A0 @ 8%
        /// Primary 10% overlay
        public static let primary10 = Color("Nym.Primary10", bundle: .module) // #5BF0A0 @ 10%
        /// Primary 22% overlay
        public static let primary22 = Color("Nym.Primary22", bundle: .module) // #5BF0A0 @ 22%
        /// Primary 40% overlay
        public static let primary40 = Color("Nym.Primary40", bundle: .module) // #5BF0A0 @ 40%

        // MARK: Alert (system-adaptive — automatically correct in light and dark mode)
        /// Alert card surface — light gray in light mode, dark gray (#2c2c2e) in dark mode.
        public static let alertSurface: Color = {
#if canImport(UIKit)
            return Color(UIColor.secondarySystemBackground)
#else
            return Color(NSColor.windowBackgroundColor)
#endif
        }()

        /// Primary text colour inside an alert — adapts to light/dark mode.
        public static let alertPrimaryText: Color = {
#if canImport(UIKit)
            return Color(UIColor.label)
#else
            return Color(NSColor.labelColor)
#endif
        }()

        /// Secondary text colour inside an alert (e.g. message body).
        public static let alertSecondaryText: Color = {
#if canImport(UIKit)
            return Color(UIColor.secondaryLabel)
#else
            return Color(NSColor.secondaryLabelColor)
#endif
        }()

        // MARK: ArcProgress (sphere + anonymous-mode arc)
        /// Sphere radial-gradient top stop — central glow body
        public static let sphereGradientTop    = Color("Nym.SphereGradientTop", bundle: .module)    // #2A2A2D
        /// Sphere radial-gradient bottom stop — outer dark falloff
        public static let sphereGradientBottom = Color("Nym.SphereGradientBottom", bundle: .module) // #090909
        /// Anonymous-mode arc fill — 5-hop progress arc colour
        public static let anonymousArc         = Color("Nym.AnonymousArc", bundle: .module)         // #8A8A90

        // MARK: Snackbar (inverse surface — light on dark app background)
        /// Snackbar card surface
        public static let snackbarSurface   = Color("Nym.SnackbarSurface", bundle: .module)  // #EEEEEE
        /// Snackbar text — near-black
        public static let snackbarText      = Color("Nym.SnackbarText", bundle: .module)     // #1C1B1F
        /// Critical snackbar background — red
        public static let snackbarCritical  = Color("Nym.SnackbarCritical", bundle: .module) // #CD2C3C
    }
}
