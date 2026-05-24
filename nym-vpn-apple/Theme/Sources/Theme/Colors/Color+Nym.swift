import SwiftUI
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

public extension Color {
    enum Nym {
        // MARK: - Brand (Figma `color/brand/*`)
        /// `brand/primary` — Spring Green CTA. D #5BF0A0 / L #1A9B61
        public static let brandPrimary       = Color("Nym.BrandPrimary", bundle: .module)
        /// `brand/primary-hover` — pointer-over. D #4AD88C / L #158551
        public static let brandPrimaryHover  = Color("Nym.BrandPrimaryHover", bundle: .module)
        /// `brand/primary-active` — pressed. D #44EE93 / L #0F6E42
        public static let brandPrimaryActive = Color("Nym.BrandPrimaryActive", bundle: .module)
        /// `brand/on-primary` — fg on brand fill. D #0A0A0A / L #FFFFFF
        public static let brandOnPrimary     = Color("Nym.BrandOnPrimary", bundle: .module)

        // MARK: - Surface (Figma `color/surface/*`)
        /// `surface/bg` — page. D #0A0A0A / L #FFFFFF
        public static let surfaceBg     = Color("Nym.SurfaceBg", bundle: .module)
        /// `surface/elev` — card / panel / modal. D #1A1A1C / L #F6F6F7
        public static let surfaceElev   = Color("Nym.SurfaceElev", bundle: .module)
        /// `surface/sunken` — input wells. D #050505 / L #EDEDEE
        public static let surfaceSunken = Color("Nym.SurfaceSunken", bundle: .module)
        /// `surface/hair` — 1px dividers. D rgba(255,255,255,0.08) / L rgba(10,10,10,0.10)
        public static let surfaceHair   = Color("Nym.SurfaceHair", bundle: .module)

        // MARK: - Text (Figma `color/text/*`)
        /// `text/primary` — body + headings. D #FFFFFF / L #0A0A0A
        public static let textPrimary   = Color("Nym.TextPrimary", bundle: .module)
        /// `text/secondary` — labels, meta. D #AEACB1 / L #5A5A60
        public static let textSecondary = Color("Nym.TextSecondary", bundle: .module)
        /// `text/tertiary` — state, hint, disabled. #8B8B90 universal
        public static let textTertiary  = Color("Nym.TextTertiary", bundle: .module)

        // MARK: - Status (Figma `color/status/*`)
        /// `status/success` — connected, ok. D #28C96C / L #1A9B61
        public static let statusSuccess = Color("Nym.StatusSuccess", bundle: .module)
        /// `status/warning` — caution. D #FFB400 / L #C77A00
        public static let statusWarning = Color("Nym.StatusWarning", bundle: .module)
        /// `status/error` — failure. #E73E14 universal
        public static let statusError   = Color("Nym.StatusError", bundle: .module)
        /// `status/info` — info, neutral accent. D #485ECA / L #3548A3
        public static let statusInfo    = Color("Nym.StatusInfo", bundle: .module)

        // MARK: - Connection (Figma `color/connection/*`)
        /// `connection/arc-track` — idle ring stroke. D rgba(255,255,255,0.15) / L rgba(10,10,10,0.12)
        public static let connectionArcTrack  = Color("Nym.ConnectionArcTrack", bundle: .module)
        /// `connection/arc-anon` — anon-mode arc fill. D rgba(139,139,144,0.60) / L rgba(107,107,112,0.60)
        public static let connectionArcAnon   = Color("Nym.ConnectionArcAnon", bundle: .module)
        /// `connection/sphere` — centre sphere base. #0A0A0A universal
        public static let connectionSphere    = Color("Nym.ConnectionSphere", bundle: .module)
        /// `connection/sphere-hi` — sphere highlight stop. #2A2A2E universal
        public static let connectionSphereHi  = Color("Nym.ConnectionSphereHi", bundle: .module)
        /// `connection/error-tint` — sphere ambient in error. rgba(231,62,20,0.08) universal
        public static let connectionErrorTint = Color("Nym.ConnectionErrorTint", bundle: .module)

        // MARK: - Illustration (Figma `color/illustration/*`)
        /// `illustration/accent` — cool blue, drawings. #A3CDFF universal
        public static let illustrationAccent = Color("Nym.IllustrationAccent", bundle: .module)

        // MARK: - Off-palette holdovers (not in Figma 21-token set)
        /// Legacy elevated surface — pre-Figma-token-cleanup
        public static let backgroundElevated = Color("Nym.BackgroundElevated", bundle: .module)
        /// Hover/pressed surface utility
        public static let backgroundHover    = Color("Nym.BackgroundHover", bundle: .module)
        /// Disabled surface — light-only
        public static let surfaceDisabled    = Color("Nym.SurfaceDisabled", bundle: .module)
        /// Top nav bar surface
        public static let navBarBackground   = Color("Nym.NavBarBackground", bundle: .module)
        /// Border / outline — no Figma equivalent (use `.surfaceHair` for hairlines)
        public static let border             = Color("Nym.Border", bundle: .module)
        /// Disabled text — Figma has only 3 text rungs; prefer `.textTertiary`
        public static let textDisabled       = Color("Nym.TextDisabled", bundle: .module)
        /// Warning surface — light-only fill
        public static let warningSurface     = Color("Nym.WarningSurface", bundle: .module)
        /// Orange — used for "expiring soon" urgency (matches `NymColor.orange` in legacy Theme)
        public static let orange             = Color(red: 0.98, green: 0.43, blue: 0.31)

        // MARK: - Raw palette holdovers
        public static let gray1   = Color("Nym.Gray1", bundle: .module)
        public static let gray2   = Color("Nym.Gray2", bundle: .module)
        public static let gray12  = Color("Nym.Gray12", bundle: .module)
        public static let icon    = Color("Nym.Icon", bundle: .module)
        public static let black   = Color("Nym.Black", bundle: .module)
        public static let white   = Color("Nym.White", bundle: .module)
        public static let white6  = Color("Nym.White6", bundle: .module)
        public static let white8  = Color("Nym.White8", bundle: .module)

        // MARK: - Primary alpha utilities (not Figma tokens)
        public static let primary8  = Color("Nym.Primary8", bundle: .module)
        public static let primary10 = Color("Nym.Primary10", bundle: .module)
        public static let primary22 = Color("Nym.Primary22", bundle: .module)
        public static let primary40 = Color("Nym.Primary40", bundle: .module)

        // MARK: - Snackbar (universal-only by design)
        public static let snackbarSurface  = Color("Nym.SnackbarSurface", bundle: .module)
        public static let snackbarText     = Color("Nym.SnackbarText", bundle: .module)
        public static let snackbarCritical = Color("Nym.SnackbarCritical", bundle: .module)

        // MARK: - Alert (system-adaptive)
        public static let alertSurface: Color = {
#if canImport(UIKit)
            return Color(UIColor.secondarySystemBackground)
#else
            return Color(NSColor.windowBackgroundColor)
#endif
        }()

        public static let alertPrimaryText: Color = {
#if canImport(UIKit)
            return Color(UIColor.label)
#else
            return Color(NSColor.labelColor)
#endif
        }()

        public static let alertSecondaryText: Color = {
#if canImport(UIKit)
            return Color(UIColor.secondaryLabel)
#else
            return Color(NSColor.secondaryLabelColor)
#endif
        }()
    }
}
